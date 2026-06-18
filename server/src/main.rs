mod ml;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::HttpRequest;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;

// Rate limiting, Replay protection, and Federation state
struct AppState {
    seen_nonces: Mutex<HashSet<String>>,
    trusted_peers: HashMap<String, Option<VerifyingKey>>, // Peer domain -> Optional Public Key
    federation_enabled: bool,
    http_client: Client,
    node_signing_key: Option<SigningKey>,
    payload_secret_key: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug)]
struct ThreatIntelPayload {
    anonymized_signature: String,
    score: f64,
    timestamp: i64,
    source_node: String,
    signature: Option<String>, // Hex-encoded ed25519 signature
}

// Client Obfuscated Payload
#[derive(Deserialize, Debug)]
struct EncodedVerifyRequest {
    data: String,
    iv: String,
}

#[derive(Deserialize, Debug)]
struct RawVerifyRequest {
    mouse_events: Vec<(f64, f64, f64)>, // x, y, timestamp
    key_events: Vec<(String, f64)>,     // key, timestamp
    #[serde(default)]
    user_agent: String,
    timestamp: i64,
    pow: Option<PoWData>,
}

#[derive(Deserialize, Debug)]
struct PoWData {
    prefix: String,
    nonce: u64,
    hash: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    score: f64,
    passed: bool,
    message: String,
}

fn verify_pow(pow: &PoWData) -> bool {
    let mut hasher = Sha256::new();
    let msg = format!("{}{}", pow.prefix, pow.nonce);
    hasher.update(msg.as_bytes());
    let result = hasher.finalize();
    let hash_hex = hex::encode(result);
    // require difficulty 3 (leading 000)
    hash_hex.starts_with("000") && hash_hex == pow.hash
}

async fn verify(
    req: HttpRequest,
    encoded_data: web::Json<EncodedVerifyRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let client_ip = req
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".into());

    // 1. Decode and Decrypt Payload
    let cipher_bytes = match base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &encoded_data.data,
    ) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Invalid data encoding.".into(),
            })
        }
    };

    let iv_bytes = match base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &encoded_data.iv,
    ) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Invalid IV encoding.".into(),
            })
        }
    };

    if iv_bytes.len() != 12 {
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Invalid IV length.".into(),
        });
    }

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&state.payload_secret_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&iv_bytes);

    let decrypted_bytes = match cipher.decrypt(nonce, cipher_bytes.as_ref()) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Failed to decrypt payload.".into(),
            })
        }
    };

    let data: RawVerifyRequest = match serde_json::from_slice(&decrypted_bytes) {
        Ok(d) => d,
        Err(_) => {
            return HttpResponse::BadRequest().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Invalid payload structure.".into(),
            })
        }
    };

    log::info!(
        "Received verification request from IP: {}, UA: {}",
        client_ip,
        data.user_agent
    );

    // 2. Replay Protection (Nonce & Timestamp)
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if (current_time - data.timestamp).abs() > 300_000 {
        // 5 minutes window
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Request timestamp invalid or expired.".into(),
        });
    }

    let pow = match &data.pow {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Missing Proof of Work.".into(),
            });
        }
    };

    let nonce_key = format!("{}-{}", pow.prefix, pow.nonce);
    let mut seen = state.seen_nonces.lock().unwrap();
    if seen.contains(&nonce_key) {
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Replay attack detected.".into(),
        });
    }
    seen.insert(nonce_key);

    // Prevent set from growing indefinitely in memory
    // A better approach would be to use a TTL cache (e.g. moka)
    // For now, we drain a portion to prevent completely wiping the history at once
    if seen.len() > 10000 {
        let keys_to_remove: Vec<String> = seen.iter().take(5000).cloned().collect();
        for key in keys_to_remove {
            seen.remove(&key);
        }
    }

    // 3. Verify Proof of Work
    if !verify_pow(pow) {
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Invalid Proof of Work.".into(),
        });
    }

    // 4. Calculate Score
    let score = calculate_score(&data);
    let passed = score > 0.7; // Increased threshold for security

    // 5. Federated Threat Sharing (Gossip Protocol)
    if !passed && state.federation_enabled {
        // Create an anonymized threat signature (hash of features)
        let features = format!("{}_{}", data.mouse_events.len(), data.key_events.len());
        let mut hasher = Sha256::new();
        hasher.update(features.as_bytes());
        let anonymized_signature = hex::encode(hasher.finalize());

        let source_node = env::var("NODE_URL").unwrap_or_else(|_| env::var("NODE_ID").unwrap_or_else(|_| "anonymous_node".into()));
        let payload_str = format!(
            "{}_{}_{}_{}",
            anonymized_signature, score, current_time, source_node
        );

        let signature = state.node_signing_key.as_ref().map(|key| {
            let sig = key.sign(payload_str.as_bytes());
            hex::encode(sig.to_bytes())
        });

        let intel = ThreatIntelPayload {
            anonymized_signature,
            score,
            timestamp: current_time,
            source_node,
            signature,
        };

        let peers = state.trusted_peers.keys().cloned().collect::<Vec<String>>();
        let client = state.http_client.clone();

        // Broadcast asynchronously without blocking the client response
        actix_web::rt::spawn(async move {
            for peer in peers {
                let url = format!("{}/api/federation/intel", peer);
                match client.post(&url).json(&intel).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        log::info!("Successfully shared threat intel with peer: {}", peer);
                    }
                    _ => {
                        log::warn!("Failed to share threat intel with peer: {}", peer);
                    }
                }
            }
        });
    }

    let message = if passed {
        "Verification successful. Human behaviour pattern confirmed."
    } else {
        "Verification failed. Automated behaviour pattern detected."
    };

    HttpResponse::Ok().json(VerifyResponse {
        score,
        passed,
        message: message.to_string(),
    })
}

// Serve sensor.js and dynamically inject the symmetric key
async fn serve_sensor_js(state: web::Data<AppState>) -> impl Responder {
    let mut file = match std::fs::File::open("../client/src/sensor.js") {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().body("Sensor script not found"),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return HttpResponse::InternalServerError().body("Failed to read sensor script");
    }

    // Convert the [u8; 32] array into a comma-separated string
    let key_string = state
        .payload_secret_key
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let injected_contents = contents.replace("__OPEN_SENTINEL_SECRET_KEY__", &key_string);

    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(injected_contents)
}

// Endpoint to receive threat intelligence from federated peers
async fn receive_threat_intel(
    intel: web::Json<ThreatIntelPayload>,
    _req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    if !state.federation_enabled {
        return HttpResponse::Forbidden().body("Federation is disabled on this node");
    }

    // 1. Peer and Cryptographic Signature Validation
    let peer_pubkey_opt = state.trusted_peers.get(&intel.source_node);

    if peer_pubkey_opt.is_none() && !state.trusted_peers.is_empty() {
        log::warn!(
            "Rejected threat intel from untrusted source: {}",
            intel.source_node
        );
        return HttpResponse::Forbidden().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Unauthorized peer.".into(),
        });
    }

    // Verify cryptographic signature if a public key is configured for this peer
    if let Some(Some(pubkey)) = peer_pubkey_opt {
        let sig_hex = match &intel.signature {
            Some(s) => s,
            None => {
                log::warn!("Missing signature from peer: {}", intel.source_node);
                return HttpResponse::Forbidden().json(VerifyResponse {
                    score: 0.0,
                    passed: false,
                    message: "Missing cryptographic signature.".into(),
                });
            }
        };

        let sig_bytes = match hex::decode(sig_hex) {
            Ok(b) => b,
            Err(_) => {
                log::warn!("Invalid signature format from peer: {}", intel.source_node);
                return HttpResponse::BadRequest().json(VerifyResponse {
                    score: 0.0,
                    passed: false,
                    message: "Invalid signature format.".into(),
                });
            }
        };

        let signature = match Signature::from_slice(&sig_bytes) {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::BadRequest().json(VerifyResponse {
                    score: 0.0,
                    passed: false,
                    message: "Malformed signature.".into(),
                });
            }
        };

        let payload_str = format!(
            "{}_{}_{}_{}",
            intel.anonymized_signature, intel.score, intel.timestamp, intel.source_node
        );
        if pubkey.verify(payload_str.as_bytes(), &signature).is_err() {
            log::warn!(
                "Cryptographic signature verification failed for peer: {}",
                intel.source_node
            );
            return HttpResponse::Forbidden().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Signature verification failed.".into(),
            });
        }
    }

    // 2. Data Validation
    if intel.score > 0.7 || intel.anonymized_signature.is_empty() {
        log::warn!(
            "Rejected invalid threat intel payload from {}",
            intel.source_node
        );
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Invalid threat signature data.".into(),
        });
    }

    log::info!(
        "Ingesting validated threat intelligence from {}: Signature {} with score {}",
        intel.source_node,
        intel.anonymized_signature,
        intel.score
    );

    // Gossip Protocol: Prevent infinite loops by tracking seen signatures
    {
        let mut seen = state.seen_nonces.lock().unwrap();
        if seen.contains(&intel.anonymized_signature) {
            return HttpResponse::Ok().json(VerifyResponse {
                score: 1.0,
                passed: true,
                message: "Threat intelligence already known.".into(),
            });
        }
        seen.insert(intel.anonymized_signature.clone());
    }

    // Write to a local threat intelligence database for model retraining
    if let Err(e) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("threat_intel.log")
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(
                f,
                "{},{},{},{}",
                intel.timestamp, intel.source_node, intel.anonymized_signature, intel.score
            )
        })
    {
        log::error!("Failed to write threat intel to disk: {}", e);
    }

    // Gossip Protocol: Forward to other trusted peers
    let peers: Vec<String> = state.trusted_peers.keys().cloned().collect();
    let client = state.http_client.clone();
    let source_node = intel.source_node.clone();
    // We must clone the payload to forward it exactly as received (including the original signature)
    let intel_payload = ThreatIntelPayload {
        anonymized_signature: intel.anonymized_signature.clone(),
        score: intel.score,
        timestamp: intel.timestamp,
        source_node: intel.source_node.clone(),
        signature: intel.signature.clone(),
    };

    actix_web::rt::spawn(async move {
        for peer in peers {
            if peer == source_node {
                continue;
            }
            let url = format!("{}/api/federation/intel", peer);
            if let Err(e) = client.post(&url).json(&intel_payload).send().await {
                log::debug!("Gossip forward failed to {}: {}", peer, e);
            }
        }
    });

    HttpResponse::Ok().json(VerifyResponse {
        score: 1.0,
        passed: true,
        message: "Threat intelligence ingested successfully.".into(),
    })
}

fn calculate_score(data: &RawVerifyRequest) -> f64 {
    // 1. Minimum Data Requirements
    if data.mouse_events.len() < 5 && data.key_events.is_empty() {
        return 0.0;
    }

    // 2. Extract features
    let mut avg_deviation = 0.0;
    let mut avg_speed = 0.0;
    let mut speed_variance = 0.0;

    if data.mouse_events.len() > 10 {
        avg_deviation = calculate_avg_deviation(&data.mouse_events);
        let (speed, var) = calculate_speed_stats(&data.mouse_events);
        avg_speed = speed;
        speed_variance = var;
    }

    let mut keystroke_interval = 200.0; // Default human-like
    let mut keystroke_variance = 50.0;

    if data.key_events.len() >= 2 {
        let (interval, var) = calculate_keystroke_stats(&data.key_events);
        keystroke_interval = interval;
        keystroke_variance = var;
    }

    // 3. Use ONNX model to predict if human or bot
    match ml::predict_bot_probability(
        avg_deviation,
        avg_speed,
        speed_variance,
        keystroke_interval,
        keystroke_variance,
    ) {
        Ok(human_prob) => human_prob,
        Err(e) => {
            log::error!("ML prediction failed: {}", e);
            0.0 // Fail secure
        }
    }
}

fn calculate_avg_deviation(events: &[(f64, f64, f64)]) -> f64 {
    if events.len() < 3 {
        return 0.0;
    }
    let start = events.first().unwrap();
    let end = events.last().unwrap();
    let a = start.1 - end.1;
    let b = end.0 - start.0;
    let c = start.0 * end.1 - end.0 * start.1;
    let denominator = (a * a + b * b).sqrt();
    if denominator == 0.0 {
        return 0.0;
    }

    let mut total_deviation = 0.0;
    for point in events.iter() {
        let distance = (a * point.0 + b * point.1 + c).abs() / denominator;
        total_deviation += distance;
    }
    total_deviation / events.len() as f64
}

fn calculate_speed_stats(events: &[(f64, f64, f64)]) -> (f64, f64) {
    let mut variances = Vec::new();
    let mut total_speed = 0.0;
    let mut count = 0;

    for window in events.windows(2) {
        let dx = window[1].0 - window[0].0;
        let dy = window[1].1 - window[0].1;
        let dt = window[1].2 - window[0].2;
        if dt > 0.0 {
            let speed = (dx * dx + dy * dy).sqrt() / dt;
            variances.push(speed);
            total_speed += speed;
            count += 1;
        }
    }
    if count == 0 {
        return (0.0, 0.0);
    }
    let avg_speed = total_speed / count as f64;
    let variance = variances
        .iter()
        .map(|s| (s - avg_speed).powi(2))
        .sum::<f64>()
        / count as f64;
    (avg_speed, variance)
}

fn calculate_keystroke_stats(events: &[(String, f64)]) -> (f64, f64) {
    let mut intervals = Vec::new();
    for window in events.windows(2) {
        intervals.push(window[1].1 - window[0].1);
    }
    if intervals.is_empty() {
        return (200.0, 50.0);
    }
    let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;
    let variance = intervals
        .iter()
        .map(|i| (i - avg_interval).powi(2))
        .sum::<f64>()
        / intervals.len() as f64;
    (avg_interval, variance)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let federation_enabled =
        env::var("FEDERATION_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";
    let trusted_peers_env = env::var("TRUSTED_PEERS").unwrap_or_else(|_| "".to_string());
    let trusted_peers_keys_env =
        env::var("TRUSTED_PEERS_PUBKEYS").unwrap_or_else(|_| "".to_string());

    let mut trusted_peers = HashMap::new();
    if !trusted_peers_env.is_empty() {
        let peers: Vec<String> = trusted_peers_env
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let pubkeys: Vec<String> = trusted_peers_keys_env
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        for (i, peer) in peers.iter().enumerate() {
            let pubkey = if i < pubkeys.len() && !pubkeys[i].is_empty() {
                if let Ok(bytes) = hex::decode(&pubkeys[i]) {
                    VerifyingKey::try_from(bytes.as_slice()).ok()
                } else {
                    None
                }
            } else {
                None
            };
            trusted_peers.insert(peer.clone(), pubkey);
        }
    }

    let node_signing_key = env::var("NODE_PRIVATE_KEY").ok().and_then(|k| {
        if let Ok(bytes) = hex::decode(k) {
            let mut array = [0u8; 32];
            if bytes.len() == 32 {
                array.copy_from_slice(&bytes);
                Some(SigningKey::from_bytes(&array))
            } else {
                None
            }
        } else {
            None
        }
    });

    let payload_secret_key = match env::var("PAYLOAD_SECRET_KEY") {
        Ok(val) => {
            let mut key = [0u8; 32];
            let bytes = hex::decode(&val).unwrap_or_else(|_| Vec::new());
            if bytes.len() == 32 {
                key.copy_from_slice(&bytes);
                key
            } else {
                log::warn!("PAYLOAD_SECRET_KEY must be a 64-character hex string (32 bytes). Falling back to default insecure key.");
                [
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                    23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
                ]
            }
        }
        Err(_) => {
            log::warn!("PAYLOAD_SECRET_KEY environment variable not set. Falling back to default insecure key. DO NOT USE IN PRODUCTION.");
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ]
        }
    };

    let app_state = web::Data::new(AppState {
        seen_nonces: Mutex::new(HashSet::new()),
        trusted_peers,
        federation_enabled,
        http_client: Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap(),
        node_signing_key,
        payload_secret_key,
    });

    log::info!("Starting OpenSentinel server at http://{}", address);
    if federation_enabled {
        log::info!(
            "Federation ENABLED. Trusted peers: {}",
            app_state.trusted_peers.len()
        );
        if app_state.node_signing_key.is_some() {
            log::info!("Node cryptographic signing is ENABLED.");
        } else {
            log::warn!(
                "Node cryptographic signing is DISABLED (NODE_PRIVATE_KEY not set or invalid)."
            );
        }
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // In production, replace with specific domains
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(web::resource("/verify").route(web::post().to(verify)))
            .service(
                web::resource("/api/federation/intel").route(web::post().to(receive_threat_intel)),
            )
            .service(web::resource("/src/sensor.js").route(web::get().to(serve_sensor_js)))
            // Serve static files from the client directory
            .service(fs::Files::new("/", "../client").index_file("index.html"))
    })
    .bind(address)?
    .run()
    .await
}

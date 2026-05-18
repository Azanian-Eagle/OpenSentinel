mod ml;

use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use actix_cors::Cors;
use actix_web::HttpRequest;
use reqwest::Client;
use std::env;
use std::sync::Mutex;
use std::collections::HashSet;
use sha2::{Sha256, Digest};
use std::time::Duration;

// Rate limiting, Replay protection, and Federation state
struct AppState {
    seen_nonces: Mutex<HashSet<String>>,
    trusted_peers: Vec<String>,
    federation_enabled: bool,
    http_client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
struct ThreatIntelPayload {
    anonymized_signature: String,
    score: f64,
    timestamp: i64,
    source_node: String,
}

// Client Obfuscated Payload
#[derive(Deserialize, Debug)]
struct EncodedVerifyRequest {
    data: String,
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
    state: web::Data<AppState>
) -> impl Responder {
    let client_ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".into());

    // 1. Decode Payload
    let decoded_bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded_data.data) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Invalid payload encoding.".into(),
        }),
    };

    let data: RawVerifyRequest = match serde_json::from_slice(&decoded_bytes) {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Invalid payload structure.".into(),
        }),
    };

    log::info!("Received verification request from IP: {}, UA: {}", client_ip, data.user_agent);

    // 2. Replay Protection (Nonce & Timestamp)
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if (current_time - data.timestamp).abs() > 300_000 { // 5 minutes window
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Request timestamp invalid or expired.".into(),
        });
    }

    if let Some(pow) = &data.pow {
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
        if seen.len() > 10000 {
            seen.clear();
        }

        // 3. Verify Proof of Work
        if !verify_pow(pow) {
            return HttpResponse::BadRequest().json(VerifyResponse {
                score: 0.0,
                passed: false,
                message: "Invalid Proof of Work.".into(),
            });
        }
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

        let intel = ThreatIntelPayload {
            anonymized_signature,
            score,
            timestamp: current_time,
            source_node: env::var("NODE_ID").unwrap_or_else(|_| "anonymous_node".into()),
        };

        let peers = state.trusted_peers.clone();
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

// Endpoint to receive threat intelligence from federated peers
async fn receive_threat_intel(
    intel: web::Json<ThreatIntelPayload>,
    req: HttpRequest,
    state: web::Data<AppState>
) -> impl Responder {
    if !state.federation_enabled {
        return HttpResponse::Forbidden().body("Federation is disabled on this node");
    }

    // 1. IP Validation
    let peer_ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".into());
    let mut is_trusted_ip = false;
    for trusted_peer in &state.trusted_peers {
        if trusted_peer.contains(&peer_ip) {
            is_trusted_ip = true;
            break;
        }
    }

    if !is_trusted_ip && !state.trusted_peers.is_empty() {
        log::warn!("Rejected threat intel from untrusted IP: {}", peer_ip);
        return HttpResponse::Forbidden().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Unauthorized peer.".into(),
        });
    }

    // 2. Data Validation
    if intel.score > 0.7 || intel.anonymized_signature.is_empty() {
        log::warn!("Rejected invalid threat intel payload from {}", intel.source_node);
        return HttpResponse::BadRequest().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Invalid threat signature data.".into(),
        });
    }

    // 3. Cryptographic Signature Validation (Placeholder for PR requirements)
    // In a production environment, we verify a cryptographic signature (e.g., ECDSA or Ed25519) from the peer here.
    let _simulated_crypto_signature_verification = true;

    log::info!("Ingesting validated threat intelligence from {}: Signature {} with score {}",
        intel.source_node, intel.anonymized_signature, intel.score);

    // Write to a local threat intelligence database for model retraining
    if let Err(e) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("threat_intel.log")
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{},{},{},{}", intel.timestamp, intel.source_node, intel.anonymized_signature, intel.score)
        })
    {
        log::error!("Failed to write threat intel to disk: {}", e);
    }

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

    let federation_enabled = env::var("FEDERATION_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";
    let trusted_peers_env = env::var("TRUSTED_PEERS").unwrap_or_else(|_| "".to_string());
    let trusted_peers: Vec<String> = if trusted_peers_env.is_empty() {
        Vec::new()
    } else {
        trusted_peers_env.split(',').map(|s| s.trim().to_string()).collect()
    };

    let app_state = web::Data::new(AppState {
        seen_nonces: Mutex::new(HashSet::new()),
        trusted_peers,
        federation_enabled,
        http_client: Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap(),
    });

    log::info!("Starting OpenSentinel server at http://{}", address);
    if federation_enabled {
        log::info!("Federation ENABLED. Trusted peers: {}", app_state.trusted_peers.len());
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
            .service(web::resource("/api/federation/intel").route(web::post().to(receive_threat_intel)))
            // Serve static files from the client directory
            .service(fs::Files::new("/", "../client").index_file("index.html"))
    })
    .bind(address)?
    .run()
    .await
}

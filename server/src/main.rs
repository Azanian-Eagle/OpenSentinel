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
use std::io::{BufRead, BufReader, Read};
use std::sync::Mutex;
use std::time::Duration;

// Rate limiting, Replay protection, and Federation state
struct AppState {
    seen_nonces: Mutex<HashSet<String>>,
    request_counts: Mutex<HashMap<String, (u32, i64)>>,
    metrics: Mutex<HashMap<&'static str, u64>>,
    trusted_peers: HashMap<String, Option<VerifyingKey>>, // Peer domain -> Optional Public Key
    federation_enabled: bool,
    http_client: Client,
    node_signing_key: Option<SigningKey>,
    payload_secret_key: [u8; 32],
    threat_intel_log_path: String,
    threat_intel_max_bytes: u64,
    rate_limit_max_requests: u32,
    rate_limit_window_ms: i64,
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    federation_enabled: bool,
}

#[derive(Serialize)]
struct ReadyResponse {
    status: &'static str,
    model_path: String,
}

#[derive(Serialize)]
struct RecentThreatIntelRecord {
    timestamp: i64,
    source_node: String,
    anonymized_signature: String,
    score: f64,
    log_source: String,
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

fn client_dir() -> String {
    env::var("OPEN_SENTINEL_CLIENT_DIR")
        .or_else(|_| env::var("CLIENT_DIR"))
        .unwrap_or_else(|_| "../client".to_string())
}

fn data_dir() -> String {
    env::var("OPEN_SENTINEL_DATA_DIR")
        .or_else(|_| env::var("DATA_DIR"))
        .unwrap_or_else(|_| "data".to_string())
}

fn allowed_origins() -> Vec<String> {
    let origins = env::var("ALLOWED_ORIGINS")
        .or_else(|_| env::var("CORS_ALLOWED_ORIGINS"))
        .unwrap_or_else(|_| "http://localhost:8080,http://127.0.0.1:8080".to_string());

    origins
        .split(',')
        .map(|origin| origin.trim())
        .filter(|origin| !origin.is_empty())
        .map(|origin| origin.to_string())
        .collect()
}

fn build_cors(allowed_origins: &[String]) -> Cors {
    let mut cors = Cors::default()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

    for origin in allowed_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}

fn rate_limit_settings() -> (u32, i64) {
    let max_requests = env::var("RATE_LIMIT_MAX_REQUESTS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(60);
    let window_seconds = env::var("RATE_LIMIT_WINDOW_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(60);

    (max_requests, window_seconds.saturating_mul(1000))
}

fn threat_intel_max_bytes() -> u64 {
    env::var("THREAT_INTEL_MAX_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5 * 1024 * 1024)
}

fn rotate_threat_intel_log(log_path: &std::path::Path, max_bytes: u64) -> std::io::Result<()> {
    let metadata = match std::fs::metadata(log_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    if metadata.len() <= max_bytes {
        return Ok(());
    }

    let rotated_path = log_path.with_extension("log.1");
    let _ = std::fs::remove_file(&rotated_path);
    std::fs::rename(log_path, rotated_path)
}

fn check_rate_limit(
    client_ip: &str,
    state: &web::Data<AppState>,
    current_time_ms: i64,
) -> Result<(), HttpResponse> {
    let mut request_counts = state.request_counts.lock().unwrap();
    let entry = request_counts
        .entry(client_ip.to_string())
        .or_insert((0, current_time_ms));

    if current_time_ms - entry.1 >= state.rate_limit_window_ms {
        *entry = (0, current_time_ms);
    }

    if entry.0 >= state.rate_limit_max_requests {
        return Err(HttpResponse::TooManyRequests().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: "Rate limit exceeded. Please retry later.".into(),
        }));
    }

    entry.0 += 1;

    if request_counts.len() > 10_000 {
        let stale_cutoff = current_time_ms.saturating_sub(state.rate_limit_window_ms * 2);
        request_counts.retain(|_, (_, last_seen)| *last_seen >= stale_cutoff);
    }

    Ok(())
}

fn increment_metric(state: &web::Data<AppState>, metric_name: &'static str) {
    let mut metrics = state.metrics.lock().unwrap();
    *metrics.entry(metric_name).or_insert(0) += 1;
}

fn metrics_text(state: &web::Data<AppState>) -> String {
    let metrics = state.metrics.lock().unwrap();

    let total_requests = metrics.get("requests_total").copied().unwrap_or(0);
    let verify_requests = metrics.get("verify_requests_total").copied().unwrap_or(0);
    let federation_ingests = metrics
        .get("federation_ingests_total")
        .copied()
        .unwrap_or(0);
    let rate_limited = metrics.get("rate_limited_total").copied().unwrap_or(0);
    let health_checks = metrics.get("health_checks_total").copied().unwrap_or(0);
    let readiness_checks = metrics.get("readiness_checks_total").copied().unwrap_or(0);

    format!(
        concat!(
            "# TYPE opensentinel_requests_total counter\n",
            "opensentinel_requests_total {}\n",
            "# TYPE opensentinel_verify_requests_total counter\n",
            "opensentinel_verify_requests_total {}\n",
            "# TYPE opensentinel_federation_ingests_total counter\n",
            "opensentinel_federation_ingests_total {}\n",
            "# TYPE opensentinel_rate_limited_total counter\n",
            "opensentinel_rate_limited_total {}\n",
            "# TYPE opensentinel_health_checks_total counter\n",
            "opensentinel_health_checks_total {}\n",
            "# TYPE opensentinel_readiness_checks_total counter\n",
            "opensentinel_readiness_checks_total {}\n"
        ),
        total_requests,
        verify_requests,
        federation_ingests,
        rate_limited,
        health_checks,
        readiness_checks
    )
}

fn append_threat_intel_record(
    state: &web::Data<AppState>,
    timestamp: i64,
    source_node: &str,
    anonymized_signature: &str,
    score: f64,
) -> std::io::Result<()> {
    let log_path = std::path::Path::new(&state.threat_intel_log_path);
    rotate_threat_intel_log(log_path, state.threat_intel_max_bytes)?;

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut file| {
            use std::io::Write;
            writeln!(
                file,
                "{},{},{},{}",
                timestamp, source_node, anonymized_signature, score
            )
        })
}

fn parse_threat_intel_record(line: &str, log_source: &str) -> Option<RecentThreatIntelRecord> {
    let mut parts = line.trim().splitn(4, ',');
    let timestamp = parts.next()?.parse::<i64>().ok()?;
    let source_node = parts.next()?.to_string();
    let anonymized_signature = parts.next()?.to_string();
    let score = parts.next()?.parse::<f64>().ok()?;

    Some(RecentThreatIntelRecord {
        timestamp,
        source_node,
        anonymized_signature,
        score,
        log_source: log_source.to_string(),
    })
}

fn read_recent_threat_intel_records(
    state: &web::Data<AppState>,
    limit: usize,
) -> std::io::Result<Vec<RecentThreatIntelRecord>> {
    let log_path = std::path::Path::new(&state.threat_intel_log_path);
    let rotated_path = log_path.with_extension("log.1");

    let mut records = Vec::new();

    for path in [rotated_path.as_path(), log_path] {
        let log_source = path.to_string_lossy().to_string();
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };

        for line in BufReader::new(file).lines().map_while(Result::ok) {
            if let Some(record) = parse_threat_intel_record(&line, &log_source) {
                records.push(record);
            }
        }
    }

    records.sort_by_key(|right| std::cmp::Reverse(right.timestamp));
    if records.len() > limit {
        records.truncate(limit);
    }

    Ok(records)
}

fn parse_payload_secret_key_value(key_value: &str) -> std::io::Result<[u8; 32]> {
    let bytes = hex::decode(key_value).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PAYLOAD_SECRET_KEY must be valid hex",
        )
    })?;

    if bytes.len() != 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PAYLOAD_SECRET_KEY must decode to exactly 32 bytes",
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

async fn healthz(state: web::Data<AppState>) -> impl Responder {
    increment_metric(&state, "requests_total");
    increment_metric(&state, "health_checks_total");
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        federation_enabled: state.federation_enabled,
    })
}

async fn readyz(state: web::Data<AppState>) -> impl Responder {
    increment_metric(&state, "requests_total");
    increment_metric(&state, "readiness_checks_total");
    match ml::get_session() {
        Ok(_) => HttpResponse::Ok().json(ReadyResponse {
            status: "ready",
            model_path: ml::model_path(),
        }),
        Err(error) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "error": error,
            "model_path": ml::model_path()
        })),
    }
}

async fn verify(
    req: HttpRequest,
    encoded_data: web::Json<EncodedVerifyRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    increment_metric(&state, "requests_total");
    increment_metric(&state, "verify_requests_total");
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

    if let Err(response) = check_rate_limit(&client_ip, &state, current_time) {
        increment_metric(&state, "rate_limited_total");
        return response;
    }

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

        let source_node = env::var("NODE_URL")
            .unwrap_or_else(|_| env::var("NODE_ID").unwrap_or_else(|_| "anonymous_node".into()));
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
    increment_metric(&state, "requests_total");
    let sensor_path = std::path::Path::new(&client_dir()).join("src/sensor.js");
    let mut file = match std::fs::File::open(sensor_path) {
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

fn parse_payload_secret_key() -> std::io::Result<[u8; 32]> {
    let key_value = env::var("PAYLOAD_SECRET_KEY").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "PAYLOAD_SECRET_KEY must be set to a 64-character hex string",
        )
    })?;

    parse_payload_secret_key_value(&key_value)
}

// Endpoint to receive threat intelligence from federated peers
async fn receive_threat_intel(
    intel: web::Json<ThreatIntelPayload>,
    _req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    increment_metric(&state, "requests_total");
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
    increment_metric(&state, "federation_ingests_total");

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
    if let Err(e) = append_threat_intel_record(
        &state,
        intel.timestamp,
        &intel.source_node,
        &intel.anonymized_signature,
        intel.score,
    ) {
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

async fn metrics(state: web::Data<AppState>) -> impl Responder {
    increment_metric(&state, "requests_total");
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics_text(&state))
}

async fn recent_threat_intel(
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    increment_metric(&state, "requests_total");

    let limit = query
        .get("limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(25)
        .clamp(1, 100);

    match read_recent_threat_intel_records(&state, limit) {
        Ok(records) => HttpResponse::Ok().json(records),
        Err(error) => HttpResponse::InternalServerError().json(VerifyResponse {
            score: 0.0,
            passed: false,
            message: format!("Failed to read threat intel records: {}", error),
        }),
    }
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
    let client_dir = client_dir();
    let allowed_origins = allowed_origins();
    let data_dir = data_dir();
    let threat_intel_log_path = std::path::Path::new(&data_dir)
        .join("threat_intel.log")
        .to_string_lossy()
        .to_string();
    let threat_intel_max_bytes = threat_intel_max_bytes();
    let (rate_limit_max_requests, rate_limit_window_ms) = rate_limit_settings();

    std::fs::create_dir_all(&data_dir)?;

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

    let payload_secret_key = parse_payload_secret_key()?;

    let app_state = web::Data::new(AppState {
        seen_nonces: Mutex::new(HashSet::new()),
        request_counts: Mutex::new(HashMap::new()),
        metrics: Mutex::new(HashMap::new()),
        trusted_peers,
        federation_enabled,
        http_client: Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap(),
        node_signing_key,
        payload_secret_key,
        threat_intel_log_path,
        threat_intel_max_bytes,
        rate_limit_max_requests,
        rate_limit_window_ms,
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
        let cors = build_cors(&allowed_origins);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(web::resource("/healthz").route(web::get().to(healthz)))
            .service(web::resource("/readyz").route(web::get().to(readyz)))
            .service(web::resource("/metrics").route(web::get().to(metrics)))
            .service(
                web::resource("/api/federation/recent").route(web::get().to(recent_threat_intel)),
            )
            .service(web::resource("/verify").route(web::post().to(verify)))
            .service(
                web::resource("/api/federation/intel").route(web::post().to(receive_threat_intel)),
            )
            .service(web::resource("/src/sensor.js").route(web::get().to(serve_sensor_js)))
            // Serve static files from the client directory
            .service(fs::Files::new("/", client_dir.clone()).index_file("index.html"))
    })
    .bind(address)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::to_bytes, test, App};

    fn test_state(federation_enabled: bool) -> web::Data<AppState> {
        web::Data::new(AppState {
            seen_nonces: Mutex::new(HashSet::new()),
            request_counts: Mutex::new(HashMap::new()),
            metrics: Mutex::new(HashMap::new()),
            trusted_peers: HashMap::new(),
            federation_enabled,
            http_client: Client::builder().build().expect("test client should build"),
            node_signing_key: None,
            payload_secret_key: [7u8; 32],
            threat_intel_log_path: "threat_intel.log".to_string(),
            threat_intel_max_bytes: 1024,
            rate_limit_max_requests: 60,
            rate_limit_window_ms: 60_000,
        })
    }

    #[actix_web::test]
    async fn parses_valid_payload_key() {
        let key = parse_payload_secret_key_value(
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        )
        .expect("valid payload key should parse");

        assert_eq!(key[0], 1);
        assert_eq!(key[31], 32);
    }

    #[actix_web::test]
    async fn rejects_invalid_payload_key_length() {
        let error = parse_payload_secret_key_value("0102").unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[actix_web::test]
    async fn verifies_pow_hash_and_prefix() {
        let prefix = "opensentinel".to_string();
        let mut nonce = 0u64;

        let pow = loop {
            let candidate = format!("{}{}", prefix, nonce);
            let hash = hex::encode(Sha256::digest(candidate.as_bytes()));

            if hash.starts_with("000") {
                break PoWData {
                    prefix: prefix.clone(),
                    nonce,
                    hash,
                };
            }

            nonce += 1;
        };

        assert!(verify_pow(&pow));
    }

    #[actix_web::test]
    async fn rate_limit_blocks_after_threshold() {
        let state = test_state(true);

        for _ in 0..60 {
            check_rate_limit("127.0.0.1", &state, 1_000).expect("first requests should pass");
        }

        let response =
            check_rate_limit("127.0.0.1", &state, 1_000).expect_err("limit should trigger");
        assert_eq!(
            response.status(),
            actix_web::http::StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[actix_web::test]
    async fn threat_intel_log_rotates_when_too_large() {
        let temp_dir = std::env::temp_dir().join("opensentinel-rotate-test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let log_path = temp_dir.join("threat_intel.log");
        std::fs::write(&log_path, "x".repeat(128)).expect("should create oversized log");

        rotate_threat_intel_log(&log_path, 16).expect("rotation should succeed");

        assert!(!log_path.exists());
        assert!(log_path.with_extension("log.1").exists());

        let _ = std::fs::remove_file(log_path.with_extension("log.1"));
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[actix_web::test]
    async fn recent_threat_intel_reports_records() {
        let temp_dir = std::env::temp_dir().join("opensentinel-recent-test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let log_path = temp_dir.join("threat_intel.log");
        std::fs::write(
            &log_path,
            "1700000000,node-a,abc123,0.3\n1700000001,node-b,def456,0.2\n",
        )
        .expect("should write log file");

        let state = web::Data::new(AppState {
            seen_nonces: Mutex::new(HashSet::new()),
            request_counts: Mutex::new(HashMap::new()),
            metrics: Mutex::new(HashMap::new()),
            trusted_peers: HashMap::new(),
            federation_enabled: true,
            http_client: Client::builder().build().expect("test client should build"),
            node_signing_key: None,
            payload_secret_key: [7u8; 32],
            threat_intel_log_path: log_path.to_string_lossy().to_string(),
            threat_intel_max_bytes: 1024,
            rate_limit_max_requests: 60,
            rate_limit_window_ms: 60_000,
        });

        let app = test::init_service(App::new().app_data(state).service(
            web::resource("/api/federation/recent").route(web::get().to(recent_threat_intel)),
        ))
        .await;

        let request = test::TestRequest::get()
            .uri("/api/federation/recent?limit=1")
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let body = to_bytes(response.into_body())
            .await
            .expect("body should read");
        let text = String::from_utf8(body.to_vec()).expect("valid utf8 body");
        assert!(text.contains("node-b"));

        let _ = std::fs::remove_file(log_path);
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[actix_web::test]
    async fn healthz_reports_ok() {
        let app = test::init_service(
            App::new()
                .app_data(test_state(true))
                .service(web::resource("/healthz").route(web::get().to(healthz))),
        )
        .await;

        let request = test::TestRequest::get().uri("/healthz").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let bytes = to_bytes(response.into_body())
            .await
            .expect("body should read");
        let body = String::from_utf8(bytes.to_vec()).expect("valid utf8 body");
        assert!(body.contains("\"status\":\"ok\""));
    }

    #[actix_web::test]
    async fn readyz_reports_ready_with_model() {
        let app = test::init_service(
            App::new()
                .app_data(test_state(true))
                .service(web::resource("/readyz").route(web::get().to(readyz))),
        )
        .await;

        let request = test::TestRequest::get().uri("/readyz").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_web::test]
    async fn metrics_reports_counters() {
        let app = test::init_service(
            App::new()
                .app_data(test_state(true))
                .service(web::resource("/metrics").route(web::get().to(metrics))),
        )
        .await;

        let request = test::TestRequest::get().uri("/metrics").to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}

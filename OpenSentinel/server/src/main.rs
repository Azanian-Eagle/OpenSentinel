use actix_files as fs;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Debug)]
struct VerifyRequest {
    mouse_events: Vec<(f64, f64, f64)>, // x, y, timestamp
    key_events: Vec<(String, f64)>,     // key, timestamp
    #[serde(default)]
    user_agent: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    score: f64,
    passed: bool,
    message: String,
}

async fn verify(data: web::Json<VerifyRequest>) -> impl Responder {
    log::info!("Received verification request");

    let score = calculate_score(&data);
    let passed = score > 0.5;

    let message = if passed {
        "Verification successful. Welcome, human."
    } else {
        "Verification failed. Bot behavior detected."
    };

    HttpResponse::Ok().json(VerifyResponse {
        score,
        passed,
        message: message.to_string(),
    })
}

fn calculate_score(data: &VerifyRequest) -> f64 {
    // Simple heuristic for PoC

    // 1. Check if we have any data
    if data.mouse_events.is_empty() && data.key_events.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;

    // 2. Mouse movement analysis
    if !data.mouse_events.is_empty() {
        // Human mouse movements are not perfectly straight.
        // We can check for variance in speed or direction, but for simplicity:
        // Just check if we have enough events for the duration.
        if data.mouse_events.len() > 10 {
            score += 0.4;
        }

        // Check time duration
        let start = data.mouse_events.first().unwrap().2;
        let end = data.mouse_events.last().unwrap().2;
        let duration = end - start;

        if duration > 1000.0 { // spent at least 1 second
            score += 0.2;
        }
    }

    // 3. Key analysis
    if !data.key_events.is_empty() {
        // Humans don't type instantly
        if data.key_events.len() > 2 {
             score += 0.2;
        }
    }

    // Cap score at 1.0
    if score > 1.0 {
        score = 1.0;
    }

    score
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    log::info!("Starting OpenSentinel server at http://{}", address);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/verify").route(web::post().to(verify)))
            // Serve static files from the client directory
            .service(fs::Files::new("/", "../client").index_file("index.html"))
    })
    .bind(address)?
    .run()
    .await
}

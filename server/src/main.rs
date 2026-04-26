mod ml;

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
    log::info!("Received verification request from UA: {}", data.user_agent);

    let score = calculate_score(&data);
    let passed = score > 0.7; // Increased threshold for security

    let message = if passed {
        "Verification successful. Human behavior pattern confirmed."
    } else {
        "Verification failed. Automated behavior pattern detected."
    };

    HttpResponse::Ok().json(VerifyResponse {
        score,
        passed,
        message: message.to_string(),
    })
}

fn calculate_score(data: &VerifyRequest) -> f64 {
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

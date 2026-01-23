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
    let mut score = 0.0;

    // 1. Minimum Data Requirements
    if data.mouse_events.len() < 5 && data.key_events.is_empty() {
        return 0.0;
    }

    // 2. Mouse Movement Analysis (Linearity & Speed)
    if data.mouse_events.len() > 10 {
        let linearity_score = analyze_mouse_linearity(&data.mouse_events);
        let speed_score = analyze_mouse_speed(&data.mouse_events);

        score += linearity_score * 0.4;
        score += speed_score * 0.2;
    } else if !data.mouse_events.is_empty() {
        // Penalty for too few mouse events if they exist
        score += 0.1;
    }

    // 3. Keystroke Dynamics
    if !data.key_events.is_empty() {
        let keystroke_score = analyze_keystrokes(&data.key_events);
        score += keystroke_score * 0.4;
    } else {
        // If only mouse used, re-weight mouse score to be out of 0.6 + base
        // But for this simple implementation, if no keys, we rely on mouse.
        // If mouse score was perfect (0.6), we need to boost it if no keys required?
        // Let's assume keys are optional but good.
        if score > 0.5 {
             score += 0.2; // Bonus for good mouse behavior without keys
        }
    }

    // Cap score at 1.0
    if score > 1.0 {
        score = 1.0;
    }

    score
}

fn analyze_mouse_linearity(events: &[(f64, f64, f64)]) -> f64 {
    // Human movements are rarely perfectly straight.
    // Calculate deviation from the line connecting start and end points.

    if events.len() < 3 {
        return 1.0; // Not enough points to judge linearity
    }

    let start = events.first().unwrap();
    let end = events.last().unwrap();

    // Line equation ax + by + c = 0
    // (y1 - y2)x + (x2 - x1)y + x1y2 - x2y1 = 0
    let a = start.1 - end.1;
    let b = end.0 - start.0;
    let c = start.0 * end.1 - end.0 * start.1;
    let denominator = (a * a + b * b).sqrt();

    if denominator == 0.0 {
        return 0.0; // Start and end are same point, suspicious for "movement" > 10 points
    }

    let mut total_deviation = 0.0;

    for point in events.iter() {
        let distance = (a * point.0 + b * point.1 + c).abs() / denominator;
        total_deviation += distance;
    }

    let avg_deviation = total_deviation / events.len() as f64;

    // If average deviation is extremely low (< 0.5px), it's likely a generated straight line.
    if avg_deviation < 0.5 {
        return 0.0; // Bot
    }

    // If it has some curve, it's good.
    1.0
}

fn analyze_mouse_speed(events: &[(f64, f64, f64)]) -> f64 {
    // Check for "teleportation" or impossible consistent speed
    // Humans accelerate and decelerate (Fitts's Law ish).

    let mut variances = Vec::new();
    let mut total_speed = 0.0;
    let mut count = 0;

    for window in events.windows(2) {
        let p1 = &window[0];
        let p2 = &window[1];

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let dt = p2.2 - p1.2;

        if dt > 0.0 {
            let speed = (dx*dx + dy*dy).sqrt() / dt;
            variances.push(speed);
            total_speed += speed;
            count += 1;
        }
    }

    if count == 0 { return 0.0; }

    let avg_speed = total_speed / count as f64;

    // Calculate variance of speed
    let variance: f64 = variances.iter().map(|s| (s - avg_speed).powi(2)).sum::<f64>() / count as f64;

    // If variance is effectively zero, speed is perfectly constant => Bot
    if variance < 0.0001 {
        return 0.0;
    }

    // Also check for superhuman speed (e.g. > 5 px/ms = 5000px/s is very fast)
    if avg_speed > 5.0 {
        return 0.0;
    }

    1.0
}

fn analyze_keystrokes(events: &[(String, f64)]) -> f64 {
    if events.len() < 2 {
        return 0.5; // Neutral
    }

    let mut intervals = Vec::new();
    for window in events.windows(2) {
        let t1 = window[0].1;
        let t2 = window[1].1;
        intervals.push(t2 - t1);
    }

    let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;

    // Superhuman typing speed check (< 50ms per key consistently)
    if avg_interval < 50.0 {
        return 0.0;
    }

    // Variance check
    let variance: f64 = intervals.iter().map(|i| (i - avg_interval).powi(2)).sum::<f64>() / intervals.len() as f64;

    // Perfectly consistent typing (variance ~ 0) => Bot
    if variance < 10.0 { // 10ms variance is still very robotic
        return 0.0;
    }

    1.0
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

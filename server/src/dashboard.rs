use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::http::header;

use crate::AppState;
use crate::read_recent_threat_intel_records;
use base64::Engine;

pub async fn dashboard(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let auth_header = req.headers().get(header::AUTHORIZATION);
    let admin_token = match &state.admin_token {
        Some(t) => t,
        None => return HttpResponse::Forbidden().body("Dashboard is disabled. Configure ADMIN_TOKEN."),
    };

    let is_authorized = if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if let Some(credentials) = auth_str.strip_prefix("Basic ") {
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(credentials) {
                    if let Ok(decoded_str) = String::from_utf8(decoded) {
                        let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
                        parts.len() == 2 && parts[0] == "admin" && parts[1] == admin_token
                    } else { false }
                } else { false }
            } else { false }
        } else { false }
    } else { false };

    if !is_authorized {
        return HttpResponse::Unauthorized()
            .append_header((header::WWW_AUTHENTICATE, "Basic realm=\"Admin Dashboard\""))
            .body("Unauthorized");
    }

    let metrics = state.metrics.lock().unwrap();
    let total_requests = metrics.get("requests_total").copied().unwrap_or(0);
    let verify_requests = metrics.get("verify_requests_total").copied().unwrap_or(0);
    let verify_success = metrics.get("verify_success_total").copied().unwrap_or(0);
    let verify_failed = metrics.get("verify_failed_total").copied().unwrap_or(0);
    let rate_limited = metrics.get("rate_limited_total").copied().unwrap_or(0);

    // Drop lock before doing IO
    drop(metrics);

    let uptime_secs = state.start_time.elapsed().as_secs();

    let records = read_recent_threat_intel_records(&state, 100).unwrap_or_default();

    // Group records by score bucket for the chart
    let mut score_buckets = vec![0; 10]; // 0.0-0.1, 0.1-0.2, etc.
    for record in &records {
        let bucket = (record.score * 10.0).floor() as usize;
        let bucket = bucket.min(9);
        score_buckets[bucket] += 1;
    }

    // Escape standard HTML injection sequences to prevent context breakage
    let records_json = serde_json::to_string(&records)
        .unwrap_or_else(|_| "[]".to_string())
        .replace("<", "\\u003c")
        .replace(">", "\\u003e")
        .replace("&", "\\u0026");
    let chart_data = serde_json::to_string(&score_buckets).unwrap_or_else(|_| "[]".to_string());

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSentinel Admin Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; background: #050505; color: #fff; margin: 0; padding: 20px; }}
        h1 {{ color: #00f3ff; text-align: center; margin-bottom: 30px; }}
        .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .card {{ background: #111; padding: 20px; border-radius: 8px; border: 1px solid #333; text-align: center; }}
        .card h3 {{ margin: 0 0 10px 0; color: #888; font-size: 0.9em; text-transform: uppercase; }}
        .card .value {{ font-size: 2em; font-weight: bold; color: #bd00ff; }}
        .chart-container {{ background: #111; padding: 20px; border-radius: 8px; border: 1px solid #333; margin-bottom: 30px; }}
        table {{ width: 100%; border-collapse: collapse; background: #111; border-radius: 8px; overflow: hidden; }}
        th, td {{ padding: 12px 15px; text-align: left; border-bottom: 1px solid #222; }}
        th {{ background: #1a1a1a; color: #00f3ff; }}
        tr:last-child td {{ border-bottom: none; }}
        .bot-score {{ color: #ff4444; }}
        .human-score {{ color: #00C851; }}
    </style>
</head>
<body>
    <h1>OpenSentinel Telemetry Dashboard</h1>

    <div class="grid">
        <div class="card">
            <h3>Uptime</h3>
            <div class="value">{uptime}s</div>
        </div>
        <div class="card">
            <h3>Total Requests</h3>
            <div class="value">{total_requests}</div>
        </div>
        <div class="card">
            <h3>Verifications</h3>
            <div class="value">{verify_requests}</div>
        </div>
        <div class="card">
            <h3>Humans (Passed)</h3>
            <div class="value" style="color: #00C851;">{verify_success}</div>
        </div>
        <div class="card">
            <h3>Bots (Failed)</h3>
            <div class="value" style="color: #ff4444;">{verify_failed}</div>
        </div>
        <div class="card">
            <h3>Rate Limited</h3>
            <div class="value" style="color: #ffbb33;">{rate_limited}</div>
        </div>
    </div>

    <div class="chart-container">
        <canvas id="scoreChart" height="80"></canvas>
    </div>

    <div style="background: #111; padding: 20px; border-radius: 8px; border: 1px solid #333;">
        <h2 style="margin-top: 0; color: #00f3ff;">Recent Threat Intelligence (Last 100)</h2>
        <div style="overflow-x: auto;">
            <table>
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>Source Node</th>
                        <th>Signature (Nonce)</th>
                        <th>Bot Score</th>
                    </tr>
                </thead>
                <tbody id="intelTable">
                </tbody>
            </table>
        </div>
    </div>

    <script>
        const chartData = {chart_data};
        const records = {records_json};

        // Render Chart
        const ctx = document.getElementById('scoreChart').getContext('2d');
        new Chart(ctx, {{
            type: 'bar',
            data: {{
                labels: ['0.0-0.1', '0.1-0.2', '0.2-0.3', '0.3-0.4', '0.4-0.5', '0.5-0.6', '0.6-0.7', '0.7-0.8', '0.8-0.9', '0.9-1.0'],
                datasets: [{{
                    label: 'Verification Scores (Closer to 1 = Bot)',
                    data: chartData,
                    backgroundColor: 'rgba(189, 0, 255, 0.5)',
                    borderColor: 'rgba(189, 0, 255, 1)',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{ beginAtZero: true, grid: {{ color: '#333' }} }},
                    x: {{ grid: {{ color: '#333' }} }}
                }},
                plugins: {{
                    legend: {{ labels: {{ color: '#fff' }} }}
                }}
            }}
        }});

        // Render Table
        const tbody = document.getElementById('intelTable');
        records.forEach(r => {{
            const tr = document.createElement('tr');

            const tdDate = document.createElement('td');
            tdDate.textContent = new Date(r.timestamp * 1000).toLocaleString();
            tr.appendChild(tdDate);

            const tdNode = document.createElement('td');
            tdNode.textContent = r.source_node;
            tr.appendChild(tdNode);

            const tdSig = document.createElement('td');
            tdSig.style.fontFamily = 'monospace';
            tdSig.style.fontSize = '0.9em';
            tdSig.style.color = '#888';
            tdSig.textContent = r.anonymized_signature.substring(0, 16) + '...';
            tr.appendChild(tdSig);

            const tdScore = document.createElement('td');
            tdScore.className = r.score > 0.5 ? 'bot-score' : 'human-score';
            const bScore = document.createElement('b');
            bScore.textContent = r.score.toFixed(2);
            tdScore.appendChild(bScore);
            tr.appendChild(tdScore);

            tbody.appendChild(tr);
        }});
    </script>
</body>
</html>
        "#,
        uptime = uptime_secs,
        total_requests = total_requests,
        verify_requests = verify_requests,
        verify_success = verify_success,
        verify_failed = verify_failed,
        rate_limited = rate_limited,
        chart_data = chart_data,
        records_json = records_json
    );

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

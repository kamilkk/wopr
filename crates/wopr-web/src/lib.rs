pub mod api;
pub mod jwt;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// A minimal HTTP/1.1 server: `/` is the dashboard, `/api/accounts/<id>` returns JSON.
/// (Dependency-light on purpose; swap in axum if you prefer — the routing is the same.)
pub async fn serve(addr: &str) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing_line(addr);
    loop {
        let (mut sock, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            let n = sock.read(&mut buf).await.unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let path = req.lines().next().and_then(|l| l.split_whitespace().nth(1)).unwrap_or("/");
            let (status, ctype, body) = route(path);
            let resp = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(resp.as_bytes()).await;
        });
    }
}

fn tracing_line(addr: &str) {
    println!("web tier on http://{addr}");
}

fn route(path: &str) -> (&'static str, &'static str, String) {
    if path == "/" {
        ("200 OK", "text/html", dashboard_html())
    } else if let Some(id) = path.strip_prefix("/api/accounts/") {
        match api::account_json(id) {
            Some(j) => ("200 OK", "application/json", j),
            None => ("404 Not Found", "application/json", r#"{"error":"not found"}"#.to_string()),
        }
    } else {
        ("404 Not Found", "text/plain", "not found".to_string())
    }
}

fn dashboard_html() -> String {
    "<!doctype html><title>WOPR</title><h1>WOPR dashboard</h1>\
     <p>Same data, two front doors: try <code>/api/accounts/00001</code>.</p>"
        .to_string()
}

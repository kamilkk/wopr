mod session_loop;

use std::sync::Arc;
use tokio::sync::Mutex;
use wopr_core::system::System;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2023").await?;
    tracing::info!("selector listening on 2023");

    let sys = Arc::new(Mutex::new(System::default()));
    loop {
        let (sock, peer) = listener.accept().await?;
        let sys = Arc::clone(&sys);
        tokio::spawn(async move {
            if let Err(e) = session_loop::run_session(sock, sys).await {
                tracing::warn!(%peer, "session ended: {e}");
            }
        });
    }
}

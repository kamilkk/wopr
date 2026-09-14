use tn3270::telnet::filter_telnet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2023").await?;
    tracing::info!("selector listening on 2023");
    loop {
        let (sock, peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = session(sock).await {
                tracing::warn!(%peer, "session ended: {e}");
            }
        });
    }
}

async fn session(mut sock: tokio::net::TcpStream) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    sock.write_all(b"\r\nWOPR  z/OS-STYLE TRAINING SYSTEM (SIMULATOR)\r\n").await?;
    sock.write_all(b"LOGON using L TSO, L CICS or L DB2.\r\n\r\nLogon Type: ").await?;
    let mut acc = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = sock.read(&mut buf).await?;
        if n == 0 { break; }                       // client hung up
        let (data, reply) = filter_telnet(&buf[..n]);
        if !reply.is_empty() { sock.write_all(&reply).await?; }
        for b in data {
            if b == b'\n' {
                let line = String::from_utf8_lossy(&acc).trim().to_string();
                acc.clear();
                sock.write_all(format!("You said: {line}\r\n").as_bytes()).await?;
            } else if b != b'\r' { acc.push(b); }
        }
    }
    Ok(())
}

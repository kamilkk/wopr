// crates/wopr-server/src/session_loop.rs
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use tn3270::telnet::filter_telnet;
use wopr_core::session::{env_banner, env_on_line, env_prompt, Reply, Session};
use wopr_core::system::System;

pub async fn run_session(mut sock: TcpStream, sys: Arc<Mutex<System>>) -> anyhow::Result<()> {
    sock.write_all(b"\r\nWOPR  z/OS-STYLE TRAINING SYSTEM (SIMULATOR)\r\n").await?;
    sock.write_all(b"LOGON using L TSO, L CICS or L DB2.\r\n\r\n").await?;

    let mut sess = Session::new();
    sock.write_all(env_prompt(&sess.env).as_bytes()).await?;

    let mut acc: Vec<u8> = Vec::new();
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
                if !feed_line(&mut sock, &sys, &mut sess, &line).await? {
                    return Ok(());                 // LOGOFF
                }
            } else if b != b'\r' {
                acc.push(b);
            }
        }
    }
    Ok(())
}

async fn feed_line(
    sock: &mut TcpStream,
    sys: &Arc<Mutex<System>>,
    sess: &mut Session,
    line: &str,
) -> anyhow::Result<bool> {
    let reply = {
        let mut sys = sys.lock().await;            // lock released at this block's end
        env_on_line(&mut sess.env, line, &mut sys, &mut sess.ctx)
    };
    match reply {
        Reply::Text(t)    => sock.write_all(t.as_bytes()).await?,
        Reply::Switch(e)  => { sess.env = e; sock.write_all(env_banner(&sess.env).as_bytes()).await?; }
        Reply::Disconnect => return Ok(false),
    }
    sock.write_all(env_prompt(&sess.env).as_bytes()).await?;
    Ok(true)
}

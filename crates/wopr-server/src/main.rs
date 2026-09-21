mod session_loop;
mod persist;

use std::sync::Arc;
use tokio::sync::Mutex;
use wopr_core::scenario::{build_system, guest_is_over_privileged, Mode, Scenario, ServiceToggles, UserSpec, Weaknesses};
use wopr_core::system::System;

fn base_users() -> Vec<UserSpec> {
    vec![
        UserSpec { userid: "IBMUSER".into(), password: "SYS1".into(), attributes: vec!["SPECIAL".into()] },
        UserSpec { userid: "GUEST".into(),   password: "GUEST".into(), attributes: vec![] },
    ]
}
fn secure_scenario() -> Scenario {
    Scenario { mode: Mode::Secure, hostname: "WOPR".into(), users: base_users(),
        services: ServiceToggles::default(), weaknesses: Weaknesses::default() }
}
fn training_scenario() -> Scenario {
    Scenario { mode: Mode::Training, hostname: "WOPR".into(), users: base_users(),
        services: ServiceToggles::default(),
        weaknesses: Weaknesses { guest_over_privileged: true, cleartext_ftp: true, ..Default::default() } }
}

/// Print, at IPL, which mode is active and which weaknesses it left switched on.
fn log_posture(sys: &System, mode: Mode) {
    tracing::info!(?mode, "scenario loaded");
    let mut active: Vec<&str> = Vec::new();
    if guest_is_over_privileged(sys) { active.push("GUEST over-privileged (OPERATIONS)"); }
    if sys.services.ftp { active.push("cleartext FTP enabled"); }
    if active.is_empty() {
        tracing::info!("security posture: hardened (no weaknesses active)");
    } else {
        for w in active { tracing::warn!("WEAKNESS ACTIVE: {w}"); }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--dump-data") {
        let sys = System::default();
        persist::dump_catalog(std::path::Path::new("data"), &sys.catalog)?;
        println!("wrote data/ from the seeded catalog");
        return Ok(());
    }

    // Select the scenario (default: secure). On real z/OS this would load scenarios/<mode>.toml.
    let scenario = if args.iter().any(|a| a == "--training") { training_scenario() } else { secure_scenario() };
    let mode = scenario.mode;
    let sys = build_system(scenario);
    log_posture(&sys, mode);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2023").await?;
    tracing::info!("selector listening on 2023");
    // DRDA listener (Phase 10): answer EXCSAT probes on 50000 so a recon tool sees a Db2.
    {
        let drda = tokio::net::TcpListener::bind("0.0.0.0:50000").await?;
        tracing::info!("DRDA listener on 50000");
        tokio::spawn(async move {
            loop {
                if let Ok((mut sock, _)) = drda.accept().await {
                    tokio::spawn(async move {
                        use tokio::io::{AsyncReadExt, AsyncWriteExt};
                        let mut buf = [0u8; 1024];
                        if let Ok(n) = sock.read(&mut buf).await {
                            if let Some(reply) = wopr_core::drda::handle_drda(&buf[..n]) {
                                let _ = sock.write_all(&reply).await;
                            }
                        }
                    });
                }
            }
        });
    }
    // Web tier (Phase 12): dashboard + REST on 8080, alongside the terminal + DRDA listeners.
    tokio::spawn(async { let _ = wopr_web::serve("0.0.0.0:8080").await; });
    let sys = Arc::new(Mutex::new(sys));
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

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

    let scenario = if args.iter().any(|a| a == "--training") { training_scenario() } else { secure_scenario() };
    let mode = scenario.mode;
    let sys = build_system(scenario);
    log_posture(&sys, mode);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2023").await?;
    tracing::info!("selector listening on 2023");
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

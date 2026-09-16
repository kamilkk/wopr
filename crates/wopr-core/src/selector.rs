use crate::logon::LogonState;
use crate::session::{Env, Environment, Reply, SessionCtx};
use crate::system::System;

pub struct Selector;
impl Environment for Selector {
    fn prompt(&self) -> &'static str { "Logon Type: " }
    fn on_line(&mut self, line: &str, _sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        let mut it = line.split_whitespace();
        match (it.next(), it.next()) {
            (Some(l), Some(app)) if l.eq_ignore_ascii_case("L") => match app.to_ascii_uppercase().as_str() {
                "TSO"          => Reply::Switch(Env::TsoLogon(LogonState::default())),
                "CICS" | "DB2" => Reply::Text(format!("{app} SELECTED (environment builds in a later phase)\r\n")),
                other          => Reply::Text(format!("DFS3649E APPLICATION {other} NOT ACTIVE\r\n")),
            },
            _ => Reply::Text("ENTER 'L TSO', 'L CICS' OR 'L DB2'\r\n".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;   // brings in Selector, Env, LogonState, Reply, SessionCtx, System
    #[test]
    fn selector_routes() {
        let (mut sys, mut ctx, mut sel) = (System::default(), SessionCtx::default(), Selector);
        // L TSO now SWITCHES into the logon panel (it returned plain text in Phase 1):
        assert!(matches!(
            sel.on_line("L TSO", &mut sys, &mut ctx),
            Reply::Switch(Env::TsoLogon(_))
        ));
        // unknown applids are still rejected in character:
        match sel.on_line("L IMS", &mut sys, &mut ctx) {
            Reply::Text(t) => assert!(t.contains("DFS3649E")),
            _ => panic!("expected rejection"),
        }
    }
}
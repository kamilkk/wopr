use crate::session::{Environment, Reply, SessionCtx};
use crate::system::System;

pub struct Selector;
impl Environment for Selector {
    fn prompt(&self) -> &'static str { "Logon Type: " }
    fn on_line(&mut self, line: &str, _sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        let mut it = line.split_whitespace();
        match (it.next(), it.next()) {
            (Some(l), Some(app)) if l.eq_ignore_ascii_case("L") => match app.to_ascii_uppercase().as_str() {
                // Each arm becomes `Reply::Switch(Env::…)` once its environment exists
                // (TSO → Phase 2, CICS → Phase 9, DB2 → Phase 10). Until then, acknowledge
                // with text so Phase 1 compiles and runs. e.g. the Phase 2 form is:
                //   "TSO" => Reply::Switch(Env::TsoLogon(crate::logon::LogonState::default())),
                "TSO" | "CICS" | "DB2" =>
                    Reply::Text(format!("{app} SELECTED (environment builds in a later phase)\r\n")),
                other =>
                    Reply::Text(format!("DFS3649E APPLICATION {other} NOT ACTIVE\r\n")),
            },
            _ => Reply::Text("ENTER 'L TSO', 'L CICS' OR 'L DB2'\r\n".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;   // brings in Selector, Environment, Reply, SessionCtx, System
    #[test]
    fn selector_routes() {
        let (mut sys, mut ctx, mut sel) = (System::default(), SessionCtx::default(), Selector);
        // known applids are accepted (each becomes a real Switch in its own phase):
        match sel.on_line("L TSO", &mut sys, &mut ctx) {
            Reply::Text(t) => assert!(t.contains("SELECTED")),
            _ => panic!("expected acceptance"),
        }
        // unknown applids are rejected in character:
        match sel.on_line("L IMS", &mut sys, &mut ctx) {
            Reply::Text(t) => assert!(t.contains("DFS3649E")),
            _ => panic!("expected rejection"),
        }
    }
}
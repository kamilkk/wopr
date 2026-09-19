use crate::racf::Logon;
use crate::session::{Environment, Reply, SessionCtx};
use crate::system::System;
use tn3270::screen::{Field, Screen};

pub struct Cics { signed_on: Option<String> }

impl Cics {
    pub fn new() -> Self { Self { signed_on: None } }

    /// Core transaction dispatch (used by the Environment impl and by tests).
    pub fn handle(&mut self, transid: &str, input: &str, sys: &mut System) -> Screen {
        match transid.to_ascii_uppercase().as_str() {
            "CESN" => self.cesn(input, sys),
            "CESF" => { self.signed_on = None; msg("DFHCE3549 Sign-off is complete.") }
            "CEMT" => self.gated("CEMT  I TASK   -- master terminal (no active tasks)"),
            "CECI" => self.gated("CECI  Command-level interpreter ready."),
            other  => msg(&format!("DFHAC2001 Transaction {other} not recognized.")),
        }
    }

    fn cesn(&mut self, input: &str, sys: &mut System) -> Screen {
        let mut it = input.split_whitespace();
        match (it.next(), it.next()) {
            (Some(user), Some(pw)) => match sys.racf.verify(user, pw) {
                Logon::Ok(u) => { self.signed_on = Some(u.userid.clone());
                    msg(&format!("DFHCE3548 Sign-on is complete for {}.", u.userid)) }
                _ => msg("DFHCE3504 The signon attempt failed."),
            },
            _ => msg("DFHCE3520 Please enter: CESN userid password"),
        }
    }

    /// A transaction that requires a signed-on identity.
    fn gated(&self, ok: &str) -> Screen {
        match &self.signed_on {
            Some(_) => msg(ok),
            None => msg("DFHAC2002 You are not signed on. Enter CESN first."),
        }
    }
}

fn msg(text: &str) -> Screen {
    Screen { fields: vec![Field { row: 0, col: 0, protected: true, hidden: false, text: text.to_string() }], cursor: (1, 0) }
}

/// Flatten a Screen to line-mode text (the Phase 7 renderer draws the same fields on a 3270).
fn screen_to_text(s: &Screen) -> String {
    let mut out = String::new();
    for f in &s.fields { out.push_str(&f.text); out.push_str("\r\n"); }
    out
}

impl Environment for Cics {
    fn prompt(&self) -> &'static str {
        if self.signed_on.is_some() { "ENTER TRANSACTION: " } else { "SIGN ON (CESN userid password): " }
    }
    fn on_line(&mut self, line: &str, sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        let line = line.trim();
        if line.is_empty() { return Reply::Text(String::new()); }
        let (transid, input) = line.split_once(' ').unwrap_or((line, ""));
        Reply::Text(screen_to_text(&self.handle(transid, input, sys)))
    }
}

mod tests {
    use super::*;
    #[test]
    fn signon_gates_transactions() {
        let mut cics = Cics::new();
        assert!(cics.signed_on.is_none());
        cics.handle("CESN", "IBMUSER SYS1", &mut System::default());  // signon
        assert_eq!(cics.signed_on.as_deref(), Some("IBMUSER"));
        // an unknown transid replies in character rather than panicking:
        let screen = cics.handle("ZZZZ", "", &mut System::default());
        assert!(screen.contains_text("DFHAC2001"));   // "transaction ZZZZ not recognized"
    }
}
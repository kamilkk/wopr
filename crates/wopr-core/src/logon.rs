use crate::audit::{EventKind, SmfEvent};
use crate::racf::Logon;
use crate::session::{Env, Environment, Reply, SessionCtx};
use crate::system::System;
use crate::tso::Tso;
use serde_json::json;

#[derive(Default)]
pub enum LogonState { #[default] AskUser, AskPass { userid: String } }

impl Environment for LogonState {
    fn prompt(&self) -> &'static str {
        match self { LogonState::AskUser => "USERID: ", LogonState::AskPass { .. } => "PASSWORD: " }
    }
    fn on_line(&mut self, line: &str, sys: &mut System, ctx: &mut SessionCtx) -> Reply {
        match std::mem::take(self) {
            LogonState::AskUser => {
                *self = LogonState::AskPass { userid: line.trim().into() };
                Reply::Text(String::new())
            }
            LogonState::AskPass { userid } => match sys.racf.verify(&userid, line.trim()) {
                Logon::Ok(u) => {
                    sys.audit.record(SmfEvent::now(EventKind::LogonOk, &u.userid, json!({})));
                    ctx.set_user(&u);
                    Reply::Switch(Env::Tso(Tso::new(u)))
                }
                Logon::BadPassword | Logon::NoUser => {
                    sys.audit.record(SmfEvent::now(
                        EventKind::LogonFail, &userid, json!({"reason": "BadPassword"})));
                    Reply::Text("IKJ56420I USERID OR PASSWORD INVALID\r\n".into())
                }
                Logon::Revoked => {
                    sys.audit.record(SmfEvent::now(
                        EventKind::LogonFail, &userid, json!({"reason": "Revoked"})));
                    Reply::Text("IKJ56429I USERID ACCESS REVOKED\r\n".into())
                }
            },
        }
    }
}

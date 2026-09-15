use crate::racf::Logon;
use crate::session::{Env, Environment, Reply, SessionCtx};
use crate::system::System;
use crate::tso::Tso;

#[derive(Default)]
pub enum LogonState { #[default] AskUser, AskPass { userid: String } }

impl Environment for LogonState {
    fn prompt(&self) -> &'static str {
        match self { LogonState::AskUser => "USERID: ", LogonState::AskPass{..} => "PASSWORD: " }
    }
    fn on_line(&mut self, line: &str, sys: &mut System, c: &mut SessionCtx) -> Reply {
        match std::mem::take(self) {
            LogonState::AskUser => { *self = LogonState::AskPass { userid: line.trim().into() };
                Reply::Text(String::new()) }
            LogonState::AskPass { userid } => match sys.racf.verify(&userid, line.trim()) {
                Logon::Ok(u) => { c.set_user(&u); Reply::Switch(Env::Tso(Tso::new(u))) }
                Logon::BadPassword | Logon::NoUser =>
                    Reply::Text("IKJ56420I USERID OR PASSWORD INVALID\r\n".into()),
                Logon::Revoked => Reply::Text("IKJ56429I USERID ACCESS REVOKED\r\n".into()),
            },
        }
    }
}

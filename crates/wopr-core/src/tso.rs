use crate::racf::User;
use crate::session::{Environment, Reply, SessionCtx};
use crate::system::System;

pub struct Tso;
impl Tso { pub fn new(_user: User) -> Self { Self } }

impl Environment for Tso {
    fn prompt(&self) -> &'static str { "READY\r\n" }
    fn on_line(&mut self, _line: &str, _sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        Reply::Text(String::new())   // Phase 3: HELP / LISTUSER / PROFILE / TIME / LOGOFF
    }
}

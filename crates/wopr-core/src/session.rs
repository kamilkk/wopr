use crate::system::System;

// Env holds each environment's value; it gains ONE variant per environment phase.
// Start with just the selector and uncomment each variant when you build its phase,
// so the code compiles at every step:
pub enum Env {
    Selector(crate::selector::Selector),
    // TsoLogon(crate::logon::LogonState),   // Phase 2
    // Tso(crate::tso::Tso),                  // Phase 3
    // Cics(crate::cics::Cics),               // Phase 9
    // Db2(crate::db2::Db2),                  // Phase 10
    // Omvs(crate::omvs::Omvs),               // Phase 11
}

pub enum Reply {
    Text(String),                   // send text, stay in this env
    Switch(Env),                    // change environment (emit its banner)
    Disconnect,                     // LOGOFF / drop the line
}

#[derive(Default)]
pub struct SessionCtx { pub user: Option<String> }   // gains set_user() in Phase 2

pub trait Environment {
    fn prompt(&self) -> &'static str;
    fn on_line(&mut self, line: &str, sys: &mut System, ctx: &mut SessionCtx) -> Reply;
}

pub struct Session { pub env: Env, pub ctx: SessionCtx }

impl Session {
    pub fn new() -> Self {
        Self { env: Env::Selector(crate::selector::Selector), ctx: SessionCtx::default() }
    }
}

// Dispatch over the enum with small free functions. Because `env` and `ctx` are
// separate fields, the server passes `&mut sess.env` and `&mut sess.ctx` at the same
// time (disjoint borrows) — no `Rc<RefCell<…>>`. Add one arm here per Env variant:
pub fn env_on_line(env: &mut Env, line: &str, sys: &mut System, ctx: &mut SessionCtx) -> Reply {
    match env { Env::Selector(e) => e.on_line(line, sys, ctx) }
}
pub fn env_prompt(env: &Env) -> &'static str {
    match env { Env::Selector(e) => e.prompt() }
}
pub fn env_banner(env: &Env) -> &'static str {
    match env { Env::Selector(_) => "" }
}

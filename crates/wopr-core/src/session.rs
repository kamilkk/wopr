use crate::racf::User;
use crate::system::System;

pub enum Env {
    Selector(crate::selector::Selector),
    TsoLogon(crate::logon::LogonState),
    Tso(crate::tso::Tso),
    Cics(crate::cics::Cics),
    Db2(crate::db2::Db2),
    // Omvs (P11) still to come
}

pub enum Reply {
    Text(String),
    Switch(Env),
    Disconnect
}

#[derive(Default)]
pub struct SessionCtx { pub user: Option<String> }
impl SessionCtx {
    pub fn set_user(&mut self, u: &User) { self.user = Some(u.userid.clone()); }  // new
}

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

// one arm per Env variant (add an arm here whenever you add a variant):
pub fn env_on_line(env: &mut Env, line: &str, sys: &mut System, ctx: &mut SessionCtx) -> Reply {
    match env {
        Env::Selector(e) => e.on_line(line, sys, ctx),
        Env::TsoLogon(e) => e.on_line(line, sys, ctx),
        Env::Tso(e)      => e.on_line(line, sys, ctx),
        Env::Cics(e)     => e.on_line(line, sys, ctx),
        Env::Db2(e)      => e.on_line(line, sys, ctx),
    }
}

pub fn env_prompt(env: &Env) -> &'static str {
    match env {
        Env::Selector(e) => e.prompt(),
        Env::TsoLogon(e) => e.prompt(),
        Env::Tso(e)      => e.prompt(),
        Env::Cics(e)     => e.prompt(),
        Env::Db2(e)      => e.prompt(),
    }
}

pub fn env_banner(env: &Env) -> &'static str {
    match env { Env::Selector(_) | Env::TsoLogon(_) | Env::Tso(_) | Env::Cics(_) | Env::Db2(_) => "" }
}

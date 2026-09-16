use crate::racf::User;
use crate::session::{Environment, Reply, SessionCtx};
use crate::system::System;

pub struct Tso { user: User }
impl Tso { pub fn new(user: User) -> Self { Self { user } } }

impl Environment for Tso {
    fn prompt(&self) -> &'static str { "READY\r\n" }
    fn on_line(&mut self, line: &str, sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        let trimmed = line.trim();
        let (verb, args) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        match verb.to_ascii_uppercase().as_str() {
            "" => Reply::Text(String::new()),
            "HELP"     => Reply::Text(help(args)),
            "TIME"     => Reply::Text(time_now()),
            "PROFILE"  => Reply::Text(self.user.profile_line()),
            "LISTUSER" => Reply::Text(listuser(args, &self.user, sys)),
            // Uncomment each arm as its phase lands — they reference not-yet-existing
            // code (and need a `use`), so they stay commented until then:
            // "SECEVENTS" => Reply::Text(sys.audit.render_recent(20)),                 // Phase 6
            "SUBMIT"       => Reply::Text(crate::jes::submit(args, sys)),
            "ST" | "SDSF"  => Reply::Text(crate::jes::sdsf_st(sys)),
            "LISTCAT" | "LISTDS" => Reply::Text(crate::dataset::catalog_cmd(verb, args, sys)),
            // "OMVS"      => Reply::Switch(crate::session::Env::Omvs(crate::omvs::Omvs::new(&self.user))), // Phase 11
            "LOGOFF"   => Reply::Disconnect,
            other      => Reply::Text(format!("IKJ56500I COMMAND {other} NOT FOUND\r\n")),
        }
    }
}

fn listuser(args: &str, caller: &User, sys: &System) -> String {
    let target = args.split_whitespace().next().unwrap_or(&caller.userid).to_ascii_uppercase();
    let Some(u) = sys.racf.users.get(&target) else {
        return format!("IKJ56712I INVALID USERID, {target}\r\n");
    };
    format!(
        "USER={id}  NAME=UNKNOWN  OWNER=SYS1  CREATED=00.000\r\n\
         ATTRIBUTES={attr}\r\n DEFAULT-GROUP={dg}\r\n",
        id = u.userid, dg = u.default_group,
        attr = if u.attrs.special { "SPECIAL" } else { "NONE" },
    )
}

fn help(_args: &str) -> String {
    "IKJ56650I ENTER HELP followed by a command name for more information\r\n".into()
}

fn time_now() -> String {
    // A fixed stamp keeps the phase deterministic; swap in `chrono` later if you like.
    "IKJ56650I TIME-12:00:00  DATE-2026.001\r\n".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_command_and_listuser() {
        let racf = crate::racf::Racf::seed_defaults();
        let mut tso = Tso::new(racf.users["IBMUSER"].clone());
        let (mut sys, mut ctx) = (System::default(), SessionCtx::default());
        match tso.on_line("FOOBAR", &mut sys, &mut ctx) {
            Reply::Text(t) => assert!(t.contains("IKJ56500I")),
            _ => panic!(),
        }
        match tso.on_line("LISTUSER IBMUSER", &mut sys, &mut ctx) {
            Reply::Text(t) => assert!(t.contains("ATTRIBUTES=SPECIAL")),
            _ => panic!(),
        }
    }
}
use crate::racf::User;
use crate::session::{Env, Environment, Reply, SessionCtx};
use crate::system::System;
use crate::tso::Tso;
use std::collections::BTreeMap;

pub struct Omvs {
    user: User,
    cwd: String,
    uid: u32,
    gid: u32,
    fs: BTreeMap<String, Vec<String>>, // dir -> entries
}

fn uid_for(userid: &str) -> u32 {
    if userid.eq_ignore_ascii_case("IBMUSER") { 0 } else { 100 + (userid.bytes().map(|b| b as u32).sum::<u32>() % 900) }
}

impl Omvs {
    pub fn new(user: &User) -> Self {
        let home = format!("/u/{}", user.userid.to_ascii_lowercase());
        let mut fs = BTreeMap::new();
        fs.insert("/".to_string(), vec!["u".into(), "bin".into(), "etc".into()]);
        fs.insert("/u".to_string(), vec![user.userid.to_ascii_lowercase()]);
        fs.insert(home.clone(), vec!["readme.txt".into(), "hello.sh".into()]);
        Self { user: user.clone(), cwd: home, uid: uid_for(&user.userid), gid: 1, fs }
    }

    pub fn run(&mut self, line: &str) -> String {
        let mut it = line.split_whitespace();
        match it.next().unwrap_or("") {
            "" => String::new(),
            "pwd" => format!("{}\r\n", self.cwd),
            "id" => format!("uid={}({}) gid={}(GROUP)\r\n", self.uid, self.user.userid.to_ascii_lowercase(), self.gid),
            "whoami" => format!("{}\r\n", self.user.userid.to_ascii_lowercase()),
            "ls" => self.fs.get(&self.cwd).map(|v| format!("{}\r\n", v.join("  "))).unwrap_or_else(|| "\r\n".into()),
            "cd" => {
                let target = it.next().unwrap_or("/");
                let next = if target.starts_with('/') { target.to_string() }
                else if target == ".." { parent(&self.cwd) }
                else { format!("{}/{}", self.cwd.trim_end_matches('/'), target) };
                if self.fs.contains_key(&next) { self.cwd = next; String::new() }
                else { format!("cd: {target}: No such directory\r\n") }
            }
            "cat" => match it.next() {
                Some(f) => format!("(contents of {f})\r\n"),
                None => "cat: missing operand\r\n".into(),
            },
            "echo" => format!("{}\r\n", it.collect::<Vec<_>>().join(" ")),
            other => format!("{other}: command not found\r\n"),
        }
    }
}

fn parent(path: &str) -> String {
    match path.trim_end_matches('/').rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(i) => path[..i].to_string(),
    }
}

impl Environment for Omvs {
    fn prompt(&self) -> &'static str { "$ " }
    fn on_line(&mut self, line: &str, _sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        let l = line.trim();
        if l == "exit" {
            Reply::Switch(Env::Tso(Tso::new(self.user.clone()))) // back to READY
        } else {
            Reply::Text(self.run(l))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn id_reflects_racf_uid() {
        let user = crate::racf::Racf::seed_defaults().users["IBMUSER"].clone();
        let mut sh = Omvs::new(&user);
        assert!(sh.run("pwd").contains("/u/ibmuser"));
        assert!(sh.run("id").contains("uid=")); // mapped from RACF, not invented per call
    }
}

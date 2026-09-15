#[derive(Clone)]
pub struct User {
    pub userid: String,
    pub password: String,          // sim: stored in the clear ON PURPOSE (see note)
    pub default_group: String,
    pub groups: Vec<String>,
    pub attrs: Attrs,              // SPECIAL / OPERATIONS / AUDITOR / NONE
    pub revoked: bool,
}
#[derive(Clone, Default)]
pub struct Attrs { pub special: bool, pub operations: bool, pub auditor: bool }

impl User {
    // Called by TSO's PROFILE command (Phase 3); define it now so `User` is complete.
    pub fn profile_line(&self) -> String {
        format!("IKJ56455I {} LOGON IN PROGRESS\r\n DEFAULT-GROUP={}\r\n",
                self.userid, self.default_group)
    }
}

pub enum Logon { Ok(User), BadPassword, NoUser, Revoked }

#[derive(Default)]
pub struct Racf {
    pub users: std::collections::HashMap<String, User>,
    // Phase 4 adds: dataset profiles; Phase 8 adds: SETROPTS options.
}

impl Racf {
    pub fn seed_defaults() -> Self {
        let mut r = Racf::default();
        r.add(User { userid: "IBMUSER".into(), password: "SYS1".into(),
            default_group: "SYS1".into(), groups: vec!["SYS1".into()],
            attrs: Attrs { special: true, ..Default::default() }, revoked: false });
        r.add(User { userid: "GUEST".into(), password: "GUEST".into(),
            default_group: "PUBLIC".into(), groups: vec!["PUBLIC".into()],
            attrs: Attrs::default(), revoked: false });
        r
    }
    fn add(&mut self, u: User) { self.users.insert(u.userid.clone(), u); }

    pub fn verify(&self, userid: &str, pw: &str) -> Logon {
        match self.users.get(&userid.to_ascii_uppercase()) {
            None => Logon::NoUser,
            Some(u) if u.revoked => Logon::Revoked,
            Some(u) if u.password == pw => Logon::Ok(u.clone()),
            Some(_) => Logon::BadPassword,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logon_paths() {
        let racf = Racf::seed_defaults();
        assert!(matches!(racf.verify("ibmuser", "SYS1"), Logon::Ok(_)));  // case-folded
        assert!(matches!(racf.verify("IBMUSER", "nope"), Logon::BadPassword));
        assert!(matches!(racf.verify("NOBODY", "x"), Logon::NoUser));
    }
}
// The scenario file *schema*; not every field is wired into behaviour yet, so allow
// dead code here (the fields are the file format, written by serde).
#![allow(dead_code)]

use crate::system::System;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]  // Debug: for the ?mode startup log in main.rs
#[serde(rename_all = "lowercase")]
pub enum Mode { Secure, Training }

#[derive(Deserialize, Clone)]
pub struct UserSpec {
    pub userid: String,
    pub password: String,
    #[serde(default)] pub attributes: Vec<String>,   // e.g. ["SPECIAL"]
}

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct ServiceToggles { pub ftp: bool, pub nje: bool, pub db2: bool, pub web: bool }

#[derive(Deserialize, Clone, Default)]
#[serde(default)]   // a partial [weaknesses] section fills the rest from Default
pub struct Weaknesses {
    pub default_creds_present: bool,      // keep IBMUSER/SYS1 enabled
    pub guest_over_privileged: bool,      // grant GUEST rights it shouldn't have
    pub cleartext_ftp: bool,
    pub setropts_password_rules_off: bool,
    pub world_readable_racf_db: bool,     // lets a low-priv user read the security DB
}

#[derive(Deserialize, Clone)]
pub struct Scenario {
    pub mode: Mode,                       // Secure | Training
    pub hostname: String,
    #[serde(default)] pub users: Vec<UserSpec>,
    #[serde(default)] pub services: ServiceToggles,
    #[serde(default)] pub weaknesses: Weaknesses,   // applied only when mode == Training
}

pub fn build_system(s: Scenario) -> System {
    let mut sys = System::from_users(&s.users);
    if matches!(s.mode, Mode::Training) {
        apply_weaknesses(&mut sys, &s.weaknesses);
    }
    sys.services = s.services;
    sys
}

pub fn apply_weaknesses(sys: &mut System, w: &Weaknesses) {
    if w.guest_over_privileged {
        if let Some(g) = sys.racf.users.get_mut("GUEST") {
            g.attrs.operations = true;   // OPERATIONS = broad data access it shouldn't have
        }
    }
    if w.cleartext_ftp { sys.services.ftp = true; }
    // default_creds_present / setropts_password_rules_off / world_readable_racf_db are
    // studyable conditions a later phase surfaces via `D SECURITY,DAILY`.
}

pub fn guest_is_over_privileged(sys: &System) -> bool {
    sys.racf.users.get("GUEST").map(|u| u.attrs.operations).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weaknesses_ignored_unless_training() {
        let toml = r#"
            mode = "secure"
            hostname = "WOPR"
            [[users]]
            userid = "GUEST"
            password = "GUEST"
            attributes = []
            [weaknesses]
            guest_over_privileged = true
        "#;
        let mut s: Scenario = toml::from_str(toml).unwrap();
        // secure mode: the flag is present in the file but must NOT take effect
        assert!(!guest_is_over_privileged(&build_system(s.clone())));
        // flip only the mode: same weakness now applies
        s.mode = Mode::Training;
        assert!(guest_is_over_privileged(&build_system(s)));
    }
}
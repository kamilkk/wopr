use crate::audit::Audit;
use crate::dataset::Catalog;
use crate::jes::Spool;
use crate::racf::{Attrs, Racf, User};
use crate::scenario::{ServiceToggles, UserSpec};

pub struct System {
    pub racf: Racf,
    pub catalog: Catalog,
    pub spool: Spool,
    pub audit: Audit,
    pub services: ServiceToggles,
}

impl Default for System {
    fn default() -> Self {
        Self {
            racf: Racf::seed_defaults(),
            catalog: Catalog::with_golden(),
            spool: Spool::default(),
            audit: Audit::default(),
            services: ServiceToggles::default()
        }
    }
}
impl System {
    pub fn from_users(users: &[UserSpec]) -> System {
        let mut racf = Racf::default();
        for u in users {
            let id = u.userid.to_ascii_uppercase();
            let has = |name: &str| u.attributes.iter().any(|a| a.eq_ignore_ascii_case(name));
            racf.users.insert(id.clone(), User {
                userid: id, password: u.password.clone(),
                default_group: "PUBLIC".into(), groups: vec!["PUBLIC".into()],
                attrs: Attrs { special: has("SPECIAL"), operations: has("OPERATIONS"), auditor: has("AUDITOR") },
                revoked: false,
            });
        }
        System { racf, catalog: Catalog::with_golden(), spool: Spool::default(),
            audit: Audit::default(), services: ServiceToggles::default() }
    }
}
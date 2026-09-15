use crate::racf::Racf;

pub struct System {
    pub racf: Racf,
    // pub catalog: crate::dataset::Catalog,    // Phase 4
    // pub spool: crate::jes::Spool,            // Phase 5
    // pub audit: crate::audit::Audit,          // Phase 6
    // pub services: crate::scenario::ServiceToggles, // Phase 8
}
impl Default for System {
    fn default() -> Self {
        Self { racf: Racf::seed_defaults() }   // a fresh system already has IBMUSER + GUEST
    }
}
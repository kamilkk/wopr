use crate::dataset::Catalog;
use crate::racf::Racf;

pub struct System {
    pub racf: Racf,
    pub catalog: Catalog,
    // pub spool: crate::jes::Spool,            // Phase 5
    // pub audit: crate::audit::Audit,          // Phase 6
    // pub services: crate::scenario::ServiceToggles, // Phase 8
}
impl Default for System {
    fn default() -> Self {
        Self {
            racf: Racf::seed_defaults(),
            catalog: Catalog::with_golden()
        }
    }
}
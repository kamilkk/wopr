use crate::dataset::Catalog;
use crate::racf::Racf;
use crate::jes::Spool;

pub struct System {
    pub racf: Racf,
    pub catalog: Catalog,
    pub spool: Spool,
    // pub audit: crate::audit::Audit,          // Phase 6
    // pub services: crate::scenario::ServiceToggles, // Phase 8
}
impl Default for System {
    fn default() -> Self {
        Self {
            racf: Racf::seed_defaults(),
            catalog: Catalog::with_golden(),
            spool: Spool::default(),
        }
    }
}
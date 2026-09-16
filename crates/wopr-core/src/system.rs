use crate::audit::Audit;
use crate::dataset::Catalog;
use crate::jes::Spool;
use crate::racf::Racf;

pub struct System {
    pub racf: Racf,
    pub catalog: Catalog,
    pub spool: Spool,
    pub audit: Audit,
    // pub services: crate::scenario::ServiceToggles, // Phase 8
}
impl Default for System {
    fn default() -> Self {
        Self {
            racf: Racf::seed_defaults(),
            catalog: Catalog::with_golden(),
            spool: Spool::default(),
            audit: Audit::default()
        }
    }
}
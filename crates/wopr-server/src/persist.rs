use std::io;
use std::path::Path;
use wopr_core::dataset::{Catalog, DsContent};

/// Write each sequential dataset's raw bytes to `dir/<dsn>` — NO translation.
/// FB records are fixed-length and stored back-to-back (file = records * LRECL),
/// exactly what the Part 1 codec expects. Never insert newlines.
pub fn dump_catalog(dir: &Path, cat: &Catalog) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, ds) in &cat.sets {
        if let DsContent::Seq(recs) = &ds.content {
            let mut bytes = Vec::new();
            for r in recs { bytes.extend_from_slice(r); }
            std::fs::write(dir.join(name), bytes)?;   // e.g. data/IBMUSER.CUST.DATA
        }
    }
    Ok(())
}

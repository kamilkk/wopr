use crate::system::System;   // listds/listcat/catalog_cmd take &System

#[derive(Debug)] pub enum DsOrg { Ps, Po }        // sequential vs partitioned
#[derive(Debug)] pub enum RecFm { F, Fb, V, Vb, U }
pub struct Dcb { pub recfm: RecFm, pub lrecl: u16, pub blksize: u16 }

pub enum DsContent {
    Seq(Vec<Vec<u8>>),                            // records, each LRECL bytes (FB)
    Pds(std::collections::BTreeMap<String, Vec<Vec<u8>>>), // member -> records
}
pub struct DataSet { pub name: String, pub dsorg: DsOrg, pub dcb: Dcb, pub content: DsContent }

#[derive(Default)]
pub struct Catalog { pub sets: std::collections::HashMap<String, DataSet> }

/// CUSTOMER-RECORD (Part 1, Phase 4): CUST-ID PIC 9(5)="00001";
/// CUST-NAME PIC X(20)="ACME WIDGETS INC"; CUST-BALANCE PIC S9(7)V99 COMP-3=+1234.56.
/// 30 data bytes, then 0x40 padding to LRECL=80. Raw EBCDIC — never translated.
pub const CUST_GOLDEN: [u8; 80] = [
    0xF0, 0xF0, 0xF0, 0xF0, 0xF1, 0xC1, 0xC3, 0xD4, 0xC5, 0x40,
    0xE6, 0xC9, 0xC4, 0xC7, 0xC5, 0xE3, 0xE2, 0x40, 0xC9, 0xD5,
    0xC3, 0x40, 0x40, 0x40, 0x40, 0x00, 0x01, 0x23, 0x45, 0x6C,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
];

impl Catalog {
    pub fn with_golden() -> Self {
        let mut c = Catalog::default();
        c.sets.insert("IBMUSER.CUST.DATA".into(), DataSet {
            name: "IBMUSER.CUST.DATA".into(),
            dsorg: DsOrg::Ps,
            dcb: Dcb { recfm: RecFm::Fb, lrecl: 80, blksize: 800 },
            content: DsContent::Seq(vec![CUST_GOLDEN.to_vec()]),
        });
        c
    }
}

pub fn listds(dsn: &str, sys: &System) -> String {
    let key = dsn.trim().trim_matches('\'');
    let Some(ds) = sys.catalog.sets.get(key) else {
        return format!("IKJ58503I DATA SET {key} NOT IN CATALOG\r\n");
    };
    format!("{key}\r\n RECFM={:?}  LRECL={}  BLKSIZE={}  DSORG={:?}\r\n",
            ds.dcb.recfm, ds.dcb.lrecl, ds.dcb.blksize, ds.dsorg)
}

pub fn listcat(sys: &System) -> String {
    let mut names: Vec<&String> = sys.catalog.sets.keys().collect();
    names.sort();
    let mut out = String::new();
    for n in names { out.push_str(&format!("NONVSAM ------- {n}\r\n")); }
    if out.is_empty() { out.push_str("IDC0001I NO ENTRIES FOUND\r\n"); }
    out
}

pub fn catalog_cmd(verb: &str, args: &str, sys: &System) -> String {
    match verb.to_ascii_uppercase().as_str() {
        "LISTCAT" => listcat(sys),
        _ => listds(args, sys),          // LISTDS 'dsn'
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn golden_dataset_dcb() {
        let sys = System::default();                 // seeds IBMUSER.CUST.DATA as FB/80
        let out = listds("'IBMUSER.CUST.DATA'", &sys);
        assert!(out.contains("RECFM=Fb") && out.contains("LRECL=80"));
        assert!(listds("NO.SUCH.DSN", &sys).contains("IKJ58503I"));

        // one 80-byte FB record: known head bytes, EBCDIC-space tail, no translation
        let ds = sys.catalog.sets.get("IBMUSER.CUST.DATA").unwrap();
        match &ds.content {
            DsContent::Seq(recs) => {
                assert_eq!(recs.len(), 1);
                assert_eq!(recs[0].len(), 80);
                assert_eq!(&recs[0][0..5], &[0xF0, 0xF0, 0xF0, 0xF0, 0xF1]); // "00001"
                assert!(recs[0][30..].iter().all(|&b| b == 0x40));          // trailing spaces
            }
            _ => panic!("expected sequential dataset"),
        }
    }
}
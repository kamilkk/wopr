//! CBSA-style REST data. In the full system these read the same datasets/Db2 tables the
//! green screens use; here a couple of demo accounts stand in.

pub fn account_json(id: &str) -> Option<String> {
    let accounts = [("00001", 1234.56_f64), ("00002", 42.00)];
    accounts
        .iter()
        .find(|(a, _)| *a == id)
        .map(|(a, b)| format!(r#"{{"id":"{a}","balance":{b:.2}}}"#))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_endpoint_reads_core() {
        let j = account_json("00001").expect("account 00001 exists");
        assert!(j.contains("balance"));
        assert!(account_json("99999").is_none());
    }
}

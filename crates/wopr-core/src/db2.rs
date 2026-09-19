use crate::session::{Environment, Reply, SessionCtx};
use crate::system::System;

pub struct Db2 {
    tables: Vec<(&'static str, &'static str)>, // (name, creator)
    grants: Vec<(String, String)>,             // (grantee, table)
}

impl Db2 {
    pub fn new() -> Self {
        Self {
            tables: vec![
                ("SYSTABLES", "SYSIBM"),
                ("SYSCOLUMNS", "SYSIBM"),
                ("ACCOUNT", "IBMUSER"),
                ("CUSTOMER", "IBMUSER"),
            ],
            grants: Vec::new(),
        }
    }

    /// A tiny SQL subset: SELECT ... FROM SYSIBM.SYSTABLES lists the catalog.
    pub fn run_sql(&self, sql: &str) -> String {
        let up = sql.trim().to_ascii_uppercase();
        if up.contains("SYSIBM.SYSTABLES") {
            let mut out = String::from("NAME        CREATOR\r\n");
            for (n, c) in &self.tables {
                out.push_str(&format!("{:<11} {}\r\n", n, c));
            }
            out.push_str(&format!("DSNE610I NUMBER OF ROWS DISPLAYED IS {}\r\n", self.tables.len()));
            out
        } else if up.starts_with("SELECT") {
            "DSNE610I NUMBER OF ROWS DISPLAYED IS 0\r\n".into()
        } else {
            "DSNE104I ONLY SELECT IS SUPPORTED IN THIS SIM\r\n".into()
        }
    }

    fn show(&self, what: &str) -> String {
        match what.to_ascii_uppercase().as_str() {
            "TABLES" | "DBS" => {
                let mut out = String::new();
                for (n, c) in &self.tables { out.push_str(&format!("{c}.{n}\r\n")); }
                out
            }
            "PRIVS" => {
                if self.grants.is_empty() { return "NO GRANTS\r\n".into(); }
                self.grants.iter().map(|(u, t)| format!("GRANT SELECT ON {t} TO {u}\r\n")).collect()
            }
            other => format!("DSNE104I SHOW {other} NOT RECOGNIZED\r\n"),
        }
    }
}

impl Environment for Db2 {
    fn prompt(&self) -> &'static str { "DSN " }
    fn on_line(&mut self, line: &str, _sys: &mut System, _ctx: &mut SessionCtx) -> Reply {
        let l = line.trim();
        let up = l.to_ascii_uppercase();
        if up.is_empty() {
            Reply::Text(String::new())
        } else if let Some(rest) = up.strip_prefix("RUN SQL ") {
            let _ = rest;
            Reply::Text(self.run_sql(&l[8..]))
        } else if let Some(rest) = up.strip_prefix("SHOW ") {
            Reply::Text(self.show(rest.trim()))
        } else if let Some(rest) = up.strip_prefix("GRANT SELECT ON ") {
            // GRANT SELECT ON <table> TO <user>
            let mut it = rest.split_whitespace();
            if let (Some(tbl), Some(_to), Some(user)) = (it.next(), it.next(), it.next()) {
                self.grants.push((user.to_string(), tbl.to_string()));
                Reply::Text("DSNE616I GRANT COMPLETE\r\n".into())
            } else {
                Reply::Text("DSNE104I USAGE: GRANT SELECT ON <TABLE> TO <USER>\r\n".into())
            }
        } else if up.starts_with("SELECT") {
            Reply::Text(self.run_sql(l))
        } else if up == "EXIT" || up == "END" {
            Reply::Disconnect
        } else {
            Reply::Text(format!("DSNE104I {l} NOT RECOGNIZED\r\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn select_systables() {
        let db2 = Db2::new();
        let out = db2.run_sql("SELECT NAME FROM SYSIBM.SYSTABLES");
        assert!(out.contains("SYSTABLES")); // the catalog lists itself
    }
}

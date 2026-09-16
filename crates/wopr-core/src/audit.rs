#[derive(Clone, Copy, Debug)]
pub enum EventKind { LogonOk, LogonFail, DsAccess, JobSubmit, PrivUse }

#[derive(Clone)]
pub struct SmfEvent {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub kind: EventKind,
    pub userid: String,
    pub detail: serde_json::Value,
}
impl SmfEvent {
    pub fn now(kind: EventKind, userid: &str, detail: serde_json::Value) -> Self {
        Self { ts: chrono::Utc::now(), kind, userid: userid.into(), detail }
    }
}

pub struct Audit { recent: std::collections::VecDeque<SmfEvent>, cap: usize }
impl Audit {
    pub fn new(cap: usize) -> Self { Self { recent: std::collections::VecDeque::new(), cap } }
    pub fn record(&mut self, e: SmfEvent) {
        tracing::info!(kind = ?e.kind, user = %e.userid, "SMF");
        if self.recent.len() == self.cap { self.recent.pop_front(); }
        self.recent.push_back(e);
    }
    pub fn render_recent(&self, n: usize) -> String {
        let start = self.recent.len().saturating_sub(n);
        let mut out = String::from("TIME       KIND        USER     DETAIL\r\n");
        for e in self.recent.iter().skip(start) {
            let detail = if e.detail.is_null() { String::new() } else { e.detail.to_string() };
            out.push_str(&format!("{:8}   {:<11} {:<8} {}\r\n",
                                  e.ts.format("%H:%M:%S"), format!("{:?}", e.kind), e.userid, detail));
        }
        out
    }
}
impl Default for Audit { fn default() -> Self { Self::new(1024) } }

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn events_render_in_order() {
        let mut a = Audit::new(64);
        a.record(SmfEvent::now(EventKind::LogonFail, "GUEST", json!({"reason":"BadPassword"})));
        a.record(SmfEvent::now(EventKind::LogonOk, "IBMUSER", json!({})));
        let out = a.render_recent(10);
        let (fail, ok) = (out.find("LogonFail").unwrap(), out.find("LogonOk").unwrap());
        assert!(fail < ok);      // oldest first
    }
}
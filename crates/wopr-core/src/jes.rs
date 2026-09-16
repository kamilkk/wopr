use crate::dataset::DsContent;
use crate::system::System;

#[derive(Debug, Clone)] pub enum Disp { Shr, Old, New, Mod }

#[derive(Debug, Clone)]
pub struct Dd { pub name: String, pub dsn: Option<String>, pub disp: Disp,
    pub sysout: Option<char>, pub dummy: bool }
#[derive(Debug, Clone)]
pub struct Step { pub name: String, pub pgm: String, pub dds: Vec<Dd> }
#[derive(Debug, Clone)]
pub struct Job  { pub id: u32, pub name: String, pub class: char, pub steps: Vec<Step> }

// Split a `//` statement body into (name, operation, operands).
fn split_stmt(rest: &str) -> (String, String, String) {
    let mut a = rest.splitn(2, char::is_whitespace);
    let name = a.next().unwrap_or("").to_string();
    let after = a.next().unwrap_or("").trim_start();
    let mut b = after.splitn(2, char::is_whitespace);
    let op = b.next().unwrap_or("").to_string();
    (name, op, b.next().unwrap_or("").trim().to_string())
}
// Pull `KEY=value` out of comma-separated operands (quotes stripped).
fn operand(ops: &str, key: &str) -> Option<String> {
    ops.split(',').find_map(|part| {
        part.trim().strip_prefix(key).and_then(|r| r.strip_prefix('='))
            .map(|v| v.trim().trim_matches('\'').to_string())
    })
}
fn parse_dd(name: &str, ops: &str) -> Dd {
    let dummy = ops.split(',').any(|p| p.trim().eq_ignore_ascii_case("DUMMY"));
    let disp = match operand(ops, "DISP").as_deref() {
        Some("OLD") => Disp::Old, Some("NEW") => Disp::New,
        Some("MOD") => Disp::Mod, _ => Disp::Shr,
    };
    Dd { name: name.to_string(), dsn: operand(ops, "DSN"),
        disp, sysout: operand(ops, "SYSOUT").and_then(|v| v.chars().next()), dummy }
}

pub fn parse_jcl(src: &str) -> anyhow::Result<Job> {
    let (mut job_name, mut class) = (String::new(), 'A');
    let (mut steps, mut cur): (Vec<Step>, Option<Step>) = (Vec::new(), None);
    for line in src.lines() {
        let line = line.trim_end();
        if line.is_empty() || !line.starts_with("//") || line.starts_with("//*") { continue; }
        let (name, op, ops) = split_stmt(&line[2..]);
        match op.to_ascii_uppercase().as_str() {
            "JOB"  => { job_name = name;
                if let Some(c) = operand(&ops, "CLASS").and_then(|s| s.chars().next()) { class = c; } }
            "EXEC" => { if let Some(s) = cur.take() { steps.push(s); }
                cur = Some(Step { name, pgm: operand(&ops, "PGM").unwrap_or_default(), dds: Vec::new() }); }
            "DD"   => { if let Some(s) = cur.as_mut() { s.dds.push(parse_dd(&name, &ops)); } }
            _ => {}
        }
    }
    if let Some(s) = cur.take() { steps.push(s); }
    if steps.is_empty() { anyhow::bail!("no EXEC steps found"); }
    Ok(Job { id: 0, name: job_name, class, steps })
}

fn find_dd<'a>(step: &'a Step, name: &str) -> Option<&'a Dd> {
    step.dds.iter().find(|d| d.name.eq_ignore_ascii_case(name))
}
fn iebgener(step: &Step, sys: &System) -> i32 {
    match (find_dd(step, "SYSUT1"), find_dd(step, "SYSUT2")) {
        (Some(inp), Some(_)) => match &inp.dsn {
            Some(dsn) if sys.catalog.sets.contains_key(dsn.trim_matches('\'')) => 0,
            _ => 12,   // SYSUT1 not in catalog
        },
        _ => 12,       // missing SYSUT1/SYSUT2
    }
}
fn idcams(_step: &Step, _sys: &mut System) -> i32 { 0 } // DEFINE/REPRO/LISTCAT land here later

pub fn run_step(step: &Step, sys: &mut System) -> i32 {
    match step.pgm.to_ascii_uppercase().as_str() {
        "IEFBR14"  => 0,
        "IEBGENER" => iebgener(step, sys),
        "IDCAMS"   => idcams(step, sys),
        _          => 12,   // unknown program -> JCL error
    }
}
pub fn run_job(job: &Job, sys: &mut System) -> i32 {
    job.steps.iter().map(|s| run_step(s, sys)).max().unwrap_or(0)  // max step RC = job RC
}

pub struct SpoolEntry { pub job: Job, pub rc: i32, pub sysout: Vec<(String, String)> }
#[derive(Default)]
pub struct Spool { pub jobs: Vec<SpoolEntry>, next_id: u32 }
impl Spool { fn next_job_id(&mut self) -> u32 { self.next_id += 1; self.next_id } }

fn read_source(sys: &System, spec: &str) -> Option<String> {
    let spec = spec.trim().trim_matches('\'');
    let (name, member) = match spec.split_once('(') {
        Some((d, r)) => (d, Some(r.trim_end_matches(')'))),
        None => (spec, None),
    };
    let ds = sys.catalog.sets.get(name)?;
    let recs = match (&ds.content, member) {
        (DsContent::Pds(m), Some(mem)) => m.get(mem)?,
        (DsContent::Seq(r), None) => r,
        _ => return None,
    };
    Some(recs.iter().map(|r| String::from_utf8_lossy(r).into_owned())
        .collect::<Vec<_>>().join("\n"))
}

pub fn submit(args: &str, sys: &mut System) -> String {
    let Some(src) = read_source(sys, args) else {
        return format!("IKJ56250I DATA SET {} NOT FOUND\r\n", args.trim());
    };
    let mut job = match parse_jcl(&src) {
        Ok(j) => j,
        Err(e) => return format!("IEFC001I INVALID JCL: {e}\r\n"),
    };
    job.id = sys.spool.next_job_id();
    let rc = run_job(&job, sys);
    let line = format!("JOB{:05} {:<8} {:<8} RC={:04}\r\n", job.id, job.name, "IBMUSER", rc);
    let sysout = vec![("JESMSGLG".to_string(), line.clone())];
    sys.spool.jobs.push(SpoolEntry { job, rc, sysout });
    line
}

pub fn sdsf_st(sys: &System) -> String {
    if sys.spool.jobs.is_empty() { return "NO JOBS ON OUTPUT QUEUE\r\n".into(); }
    let mut out = String::from("JOBNAME  JOBID    OWNER    RC\r\n");
    for e in &sys.spool.jobs {
        out.push_str(&format!("{:<8} JOB{:05} {:<8} {:04}\r\n", e.job.name, e.job.id, "IBMUSER", e.rc));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn iebgener_runs_rc0() {
        let src = "//RUNIT JOB (ACCT),'DEMO',CLASS=A\n\
                   //STEP1 EXEC PGM=IEBGENER\n\
                   //SYSUT1 DD DSN=IBMUSER.CUST.DATA,DISP=SHR\n\
                   //SYSUT2 DD SYSOUT=*\n//SYSIN DD DUMMY\n";
        let job = parse_jcl(src).expect("parse");
        assert_eq!(job.steps.len(), 1);
        let mut sys = System::default();
        assert_eq!(run_job(&job, &mut sys), 0);
    }
}
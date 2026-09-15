pub struct IplState {
    pub clpa: bool, pub hostname: String, pub confirmed: bool,
    pub mfa_pin: Option<String>,
}
// "R 01,CLPA" -> clpa=true ; "R 05,MYLPAR" -> hostname ; "R 03,Y" -> confirmed
// "D IPLINFO", "D SERVICES", "D R,L" -> display outstanding replies / state

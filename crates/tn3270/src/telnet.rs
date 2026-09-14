// crates/tn3270/src/telnet.rs
pub const IAC: u8 = 255;
pub const DONT: u8 = 254; pub const DO: u8 = 253;
pub const WONT: u8 = 252; pub const WILL: u8 = 251;

/// Pull telnet commands out of `raw`, returning the plain bytes and any reply to send.
pub fn filter_telnet(raw: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (mut data, mut reply) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == IAC && i + 1 < raw.len() {
            match raw[i + 1] {
                IAC => { data.push(IAC); i += 2; }
                DO | DONT if i + 2 < raw.len() => {          // refuse to DO anything
                    reply.extend_from_slice(&[IAC, WONT, raw[i + 2]]); i += 3;
                }
                WILL | WONT if i + 2 < raw.len() => {        // tell peer DONT
                    reply.extend_from_slice(&[IAC, DONT, raw[i + 2]]); i += 3;
                }
                _ => { i += 2; }
            }
        } else { data.push(raw[i]); i += 1; }
    }
    (data, reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn telnet_strip() {
        let raw = [IAC, DO, 24, b'L', b' ', b'T', b'S', b'O', b'\r', b'\n'];
        let (data, reply) = filter_telnet(&raw);
        assert_eq!(data, b"L TSO\r\n");
        assert_eq!(reply, [IAC, WONT, 24]);     // we refuse TERMINAL-TYPE in line mode
    }
}
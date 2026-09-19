//! A DRDA listener *stub*: enough to answer an EXCSAT probe with a server name so a recon
//! tool reports a plausible Db2. Not a real DRDA stack.
//!
//! A DRDA DSS is: 2-byte length, a format byte (0xD0), a 2-byte request-correlation id,
//! then DDM objects (each: 2-byte length + 2-byte code point + data). EXCSAT = 0x1041;
//! we answer with EXCSATRD = 0x1443 carrying SRVNAM (0x116D) = "WOPR".

pub fn handle_drda(frame: &[u8]) -> Option<Vec<u8>> {
    // Recognize an EXCSAT code point (0x1041) anywhere in the request.
    if !frame.windows(2).any(|w| w == [0x10, 0x41]) {
        return None;
    }
    let srvnam = b"WOPR";

    // SRVNAM object: len(2) + codepoint(2) + data
    let mut srvobj = Vec::new();
    srvobj.extend_from_slice(&((4 + srvnam.len()) as u16).to_be_bytes());
    srvobj.extend_from_slice(&[0x11, 0x6D]); // SRVNAM
    srvobj.extend_from_slice(srvnam);

    // EXCSATRD object wrapping SRVNAM
    let mut exc = Vec::new();
    exc.extend_from_slice(&((4 + srvobj.len()) as u16).to_be_bytes());
    exc.extend_from_slice(&[0x14, 0x43]); // EXCSATRD
    exc.extend_from_slice(&srvobj);

    // DSS header: len(2), format(0xD0), reply flag(0x01), correlation id(2)
    let mut dss = Vec::new();
    dss.extend_from_slice(&((6 + exc.len()) as u16).to_be_bytes());
    dss.push(0xD0);
    dss.push(0x01);
    dss.extend_from_slice(&[0x00, 0x01]);
    dss.extend_from_slice(&exc);
    Some(dss)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_excsat() -> Vec<u8> {
        // minimal DSS carrying an EXCSAT (0x1041) code point
        vec![0x00, 0x0A, 0xD0, 0x01, 0x00, 0x01, 0x00, 0x04, 0x10, 0x41]
    }
    #[test]
    fn excsat_gets_a_reply() {
        let reply = handle_drda(&sample_excsat()).expect("EXCSAT should be answered");
        assert!(reply.windows(4).any(|w| w == b"WOPR")); // SRVNAM present
    }
}

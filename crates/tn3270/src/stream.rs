use crate::ebcdic::Ebcdic;
use crate::screen::Screen;

// outbound 3270 commands
pub const CMD_WRITE: u8 = 0xF1; pub const CMD_ERASE_WRITE: u8 = 0xF5; pub const CMD_ERASE_WRITE_ALT: u8 = 0x7E;
pub const CMD_READ_BUFFER: u8 = 0xF2; pub const CMD_READ_MODIFIED: u8 = 0xF6;
// orders
pub const SF: u8 = 0x1D; pub const SBA: u8 = 0x11; pub const IC: u8 = 0x13; pub const RA: u8 = 0x3C; pub const SFE: u8 = 0x29;
// AIDs (inbound, first byte of a Read Modified)
pub const AID_ENTER: u8 = 0x7D; pub const AID_CLEAR: u8 = 0x6D; pub const AID_PA1: u8 = 0x6C;

pub const ADDR: [u8; 64] = [
    0x40,0xC1,0xC2,0xC3,0xC4,0xC5,0xC6,0xC7,0xC8,0xC9,0x4A,0x4B,0x4C,0x4D,0x4E,0x4F,
    0x50,0xD1,0xD2,0xD3,0xD4,0xD5,0xD6,0xD7,0xD8,0xD9,0x5A,0x5B,0x5C,0x5D,0x5E,0x5F,
    0x60,0x61,0xE2,0xE3,0xE4,0xE5,0xE6,0xE7,0xE8,0xE9,0x6A,0x6B,0x6C,0x6D,0x6E,0x6F,
    0xF0,0xF1,0xF2,0xF3,0xF4,0xF5,0xF6,0xF7,0xF8,0xF9,0x7A,0x7B,0x7C,0x7D,0x7E,0x7F,
];
pub fn enc_addr(pos: u16) -> [u8; 2] { [ADDR[((pos >> 6) & 0x3F) as usize], ADDR[(pos & 0x3F) as usize]] }
pub fn dec_addr(hi: u8, lo: u8) -> u16 {
    let h = ADDR.iter().position(|&b| b == hi).unwrap_or(0) as u16;
    let l = ADDR.iter().position(|&b| b == lo).unwrap_or(0) as u16;
    (h << 6) | l
}
pub fn rc_to_pos(row: u16, col: u16) -> u16 { row * 80 + col }   // model 2: 24x80

/// Field-attribute byte after SF: protected/unprotected, display/non-display, high bits
/// set so it lands as a graphic. (MDT/numeric bits omitted for the sim.)
pub fn attr_byte(protected: bool, hidden: bool) -> u8 {
    let mut a = 0xC0;
    if protected { a |= 0x20; }
    if hidden { a |= 0x0C; }   // non-display
    a
}

pub fn render_3270(s: &Screen, ebcdic: &Ebcdic) -> Vec<u8> {
    let mut b = vec![CMD_ERASE_WRITE, 0xC3];             // WCC: reset + keyboard restore
    for f in &s.fields {
        b.push(SBA); b.extend_from_slice(&enc_addr(rc_to_pos(f.row, f.col)));
        b.push(SF);  b.push(attr_byte(f.protected, f.hidden));
        b.extend(ebcdic.to_bytes(&f.text));
    }
    b.push(IC); b.extend_from_slice(&enc_addr(rc_to_pos(s.cursor.0, s.cursor.1)));
    b                                                    // caller frames with IAC EOR
}

pub struct Inbound { pub aid: u8, pub cursor: u16, pub fields: Vec<(u16, String)> }

pub fn parse_inbound(bytes: &[u8], ebcdic: &Ebcdic) -> Inbound {
    if bytes.is_empty() { return Inbound { aid: 0, cursor: 0, fields: Vec::new() }; }
    let aid = bytes[0];
    let cursor = if bytes.len() >= 3 { dec_addr(bytes[1], bytes[2]) } else { 0 };
    let mut fields = Vec::new();
    let mut i = 3;
    while i < bytes.len() {
        if bytes[i] == SBA && i + 2 < bytes.len() {
            let pos = dec_addr(bytes[i + 1], bytes[i + 2]);
            i += 3;
            let start = i;
            while i < bytes.len() && bytes[i] != SBA { i += 1; }
            fields.push((pos, ebcdic.to_string(&bytes[start..i])));
        } else { i += 1; }
    }
    Inbound { aid, cursor, fields }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::Screen;

    #[test]
    fn sba_addresses_row1_col0() {
        // pos = 1*80 + 0 = 80 -> high 6 bits = 1, low 6 bits = 80 & 0x3F = 16
        assert_eq!(enc_addr(rc_to_pos(1, 0)), [ADDR[1], ADDR[16]]);
        assert_eq!(enc_addr(0), [ADDR[0], ADDR[0]]);
    }

    #[test]
    fn addr_roundtrip() {
        for pos in [0u16, 1, 79, 80, 959, 1000, 1919] {
            let e = enc_addr(pos);
            assert_eq!(dec_addr(e[0], e[1]), pos);
        }
    }

    #[test]
    fn inbound_roundtrip() {
        let eb = Ebcdic::cp037();
        let out = render_3270(&Screen::logon(), &eb);      // host writes the panel
        assert_eq!(out[0], CMD_ERASE_WRITE);
        assert!(out.contains(&SF) && out.contains(&IC));

        // terminal replies: Enter, cursor, one modified field (userid) = "IBMUSER"
        let userid_pos = rc_to_pos(2, 14);
        let mut rm = vec![AID_ENTER];
        rm.extend_from_slice(&enc_addr(rc_to_pos(2, 14)));
        rm.push(SBA); rm.extend_from_slice(&enc_addr(userid_pos));
        rm.extend(eb.to_bytes("IBMUSER"));

        let inbound = parse_inbound(&rm, &eb);
        assert_eq!(inbound.aid, AID_ENTER);
        assert_eq!(inbound.fields.len(), 1);
        assert_eq!(inbound.fields[0].0, userid_pos);
        assert_eq!(inbound.fields[0].1, "IBMUSER");
    }
}


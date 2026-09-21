//! A tiny JWT "lab" — enough to demonstrate the `alg=none` attack, without pulling a JWT
//! crate. A real service must NEVER accept `alg=none`; the training lab does, on purpose,
//! so learners can see why it's dangerous.

pub struct JwtConfig {
    pub allow_alg_none: bool, // secure: false; training lab: true
}

fn b64url_decode(s: &str) -> Vec<u8> {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut idx = [255u8; 256];
    let mut i = 0;
    while i < 64 { idx[A[i] as usize] = i as u8; i += 1; }
    let (mut out, mut buf, mut bits) = (Vec::new(), 0u32, 0u32);
    for &c in s.as_bytes() {
        let v = idx[c as usize];
        if v == 255 { continue; }
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 { bits -= 8; out.push((buf >> bits) as u8); }
    }
    out
}

pub fn b64url_encode(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        for k in 0..(chunk.len() + 1) {
            out.push(A[((n >> (18 - 6 * k)) & 0x3F) as usize] as char);
        }
    }
    out // no padding (JWT style)
}

/// Validate a `header.payload.signature` token. Returns whether it's accepted.
pub fn validate(cfg: &JwtConfig, token: &str) -> bool {
    let mut parts = token.split('.');
    let (Some(h), Some(_p)) = (parts.next(), parts.next()) else { return false };
    let sig = parts.next().unwrap_or("");
    let header = b64url_decode(h);
    let alg = serde_json::from_slice::<serde_json::Value>(&header)
        .ok()
        .and_then(|v| v.get("alg").and_then(|a| a.as_str()).map(str::to_string));
    match alg.as_deref() {
        Some("none") => cfg.allow_alg_none, // THE VULN: unsigned token accepted only if allowed
        Some("HS256") => !sig.is_empty(),   // sim: a present signature is treated as valid
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn alg_none_token() -> String {
        format!("{}.{}.",
                b64url_encode(br#"{"alg":"none","typ":"JWT"}"#),
                b64url_encode(br#"{"sub":"admin"}"#))
    }
    #[test]
    fn alg_none_rejected_in_secure_mode() {
        let tok = alg_none_token();
        assert!(!validate(&JwtConfig { allow_alg_none: false }, &tok)); // secure: reject
        assert!( validate(&JwtConfig { allow_alg_none: true }, &tok));  // training: accepted on purpose
    }
}

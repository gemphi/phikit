//! Minimal base64 encode/decode helpers used for embedding buffer data.
//!
//! These avoid an external dependency for the small amount of base64 work the
//! exporter needs (data URIs and GLB string round-tripping).

/// Simple base64 encoding
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            1 => [chunk[0], 0, 0],
            2 => [chunk[0], chunk[1], 0],
            3 => [chunk[0], chunk[1], chunk[2]],
            _ => unreachable!(),
        };

        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);

        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Simple base64 decoding
pub(crate) fn base64_decode(data: &str) -> std::result::Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let data: String = data.chars().filter(|c| !c.is_whitespace()).collect();
    let data = data.trim_end_matches('=');

    let mut result = Vec::with_capacity(data.len() * 3 / 4);

    for chunk in data.as_bytes().chunks(4) {
        let mut n: u32 = 0;
        let mut valid_chars = 0;

        for (i, &c) in chunk.iter().enumerate() {
            let c = match c {
                b'-' => b'+',
                b'_' => b'/',
                _ => c,
            };
            if let Some(pos) = ALPHABET.iter().position(|&x| x == c) {
                n |= (pos as u32) << (18 - i * 6);
                valid_chars += 1;
            }
        }

        if valid_chars > 1 {
            result.push((n >> 16) as u8);
        }
        if valid_chars > 2 {
            result.push(((n >> 8) & 0xFF) as u8);
        }
        if valid_chars > 3 {
            result.push((n & 0xFF) as u8);
        }
    }

    Ok(result)
}

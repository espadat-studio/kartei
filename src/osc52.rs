const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn sequence(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes()))
}

fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            out.push(match i <= chunk.len() {
                true => char::from(ALPHABET[(n >> (18 - 6 * i) & 63) as usize]),
                false => '=',
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{base64, sequence};

    #[test]
    fn base64_matches_rfc_4648_vectors() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
            ("Grüße 日本", "R3LDvMOfZSDml6XmnKw="),
        ] {
            assert_eq!(base64(input.as_bytes()), expected, "{input:?}");
        }
    }

    #[test]
    fn sequence_wraps_the_base64_in_osc_52() {
        assert_eq!(sequence("foo"), "\x1b]52;c;Zm9v\x07");
    }
}

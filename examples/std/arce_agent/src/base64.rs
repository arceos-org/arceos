// Minimal base64 encoder — avoids pulling in an external crate.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode_base64(input: &[u8]) -> String {
    let mut output = String::with_capacity((input.len() + 2) / 3 * 4);
    let chunks = input.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        output.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        output.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            output.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

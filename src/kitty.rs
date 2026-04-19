use crate::video::RgbFrame;

const CHUNK: usize = 4096;
const IMAGE_ID: u32 = 1;

/// Encode a frame using the Kitty Graphics Protocol.
/// Uses stable image ID so old frames can be deleted cleanly.
/// cols/rows: scale to fill that many character cells.
pub fn encode(frame: &RgbFrame, cols: u16, rows: u16, in_tmux: bool) -> String {
    let b64 = base64(&frame.pixels);
    let n_chunks = (b64.len() + CHUNK - 1) / CHUNK;
    let mut out = String::with_capacity(b64.len() + n_chunks * 80);

    for (i, chunk) in b64.as_bytes().chunks(CHUNK).enumerate() {
        let s = unsafe { std::str::from_utf8_unchecked(chunk) };
        let more = u8::from(i + 1 < n_chunks);
        // q=2 suppresses all terminal responses (prevents "Gi=1;OK" leaking as text)
        let seq = if i == 0 {
            format!(
                "\x1b_Ga=T,f=24,q=2,i={IMAGE_ID},s={},v={},c={},r={},m={};{}\x1b\\",
                frame.width, frame.height, cols, rows, more, s
            )
        } else {
            format!("\x1b_Gm={};{}\x1b\\", more, s)
        };
        out.push_str(&wrap(seq, in_tmux));
    }
    out
}

/// Delete the image with our stable ID from the terminal's image buffer.
pub fn delete(in_tmux: bool) -> String {
    wrap(format!("\x1b_Ga=d,q=2,d=I,i={IMAGE_ID}\x1b\\"), in_tmux)
}

fn wrap(seq: String, in_tmux: bool) -> String {
    if in_tmux {
        let escaped = seq.replace('\x1b', "\x1b\x1b");
        format!("\x1bPtmux;{}\x1b\\", escaped)
    } else {
        seq
    }
}

fn base64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for c in data.chunks(3) {
        let b0 = c[0] as u32;
        let b1 = c.get(1).copied().unwrap_or(0) as u32;
        let b2 = c.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if c.len() > 1 { T[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if c.len() > 2 { T[(n & 63) as usize] as char } else { '=' });
    }
    out
}

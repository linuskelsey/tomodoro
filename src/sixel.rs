use std::collections::HashMap;
use crate::video::RgbFrame;

/// Encode a single frame as a sixel string.
/// Colors are quantized to ≤256 palette entries.
/// The output includes the DCS introducer and ST terminator.
pub fn encode(frame: &RgbFrame) -> String {
    let (palette, indices) = build_palette(frame);

    let w = frame.width;
    let h = frame.height;

    // Estimate capacity to avoid many reallocations
    let mut out = String::with_capacity(w * h / 2);

    // DCS + sixel introducer. "1;1 sets 1:1 pixel aspect ratio.
    out.push_str("\x1bPq\"1;1;");
    out.push_str(&w.to_string());
    out.push(';');
    out.push_str(&h.to_string());

    // Define palette (colors in 0-100 percent range)
    for (i, &(r, g, b)) in palette.iter().enumerate() {
        let r100 = (r as u32 * 100 / 255) as u8;
        let g100 = (g as u32 * 100 / 255) as u8;
        let b100 = (b as u32 * 100 / 255) as u8;
        out.push_str(&format!("#{};2;{};{};{}", i, r100, g100, b100));
    }

    let num_bands = (h + 5) / 6;

    for band in 0..num_bands {
        let row_start = band * 6;
        let row_end = (row_start + 6).min(h);
        let rows_in_band = row_end - row_start;

        let mut first_in_band = true;

        for color_idx in 0..palette.len() {
            // Build the sixel chars for this color across the band
            let mut chars: Vec<u8> = Vec::with_capacity(w);
            let mut has_pixels = false;

            for col in 0..w {
                let mut sixel_byte: u8 = 0;
                for row_offset in 0..rows_in_band {
                    let row = row_start + row_offset;
                    if indices[row * w + col] == color_idx as u8 {
                        sixel_byte |= 1 << row_offset;
                        has_pixels = true;
                    }
                }
                chars.push(sixel_byte + 0x3F);
            }

            if !has_pixels {
                continue;
            }

            if !first_in_band {
                out.push('$'); // carriage return within band
            }
            first_in_band = false;

            out.push('#');
            out.push_str(&color_idx.to_string());
            rle_append(&mut out, &chars);
        }

        out.push('-'); // advance to next band
    }

    // String terminator
    out.push_str("\x1b\\");
    out
}

fn rle_append(out: &mut String, bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let mut count = 1;
        while i + count < bytes.len() && bytes[i + count] == b && count < 255 {
            count += 1;
        }
        if count > 3 {
            out.push('!');
            out.push_str(&count.to_string());
            out.push(b as char);
        } else {
            for _ in 0..count {
                out.push(b as char);
            }
        }
        i += count;
    }
}

fn build_palette(frame: &RgbFrame) -> (Vec<(u8, u8, u8)>, Vec<u8>) {
    let w = frame.width;
    let h = frame.height;

    // Try exact color palette first
    let mut color_map: HashMap<(u8, u8, u8), u8> = HashMap::new();
    let mut palette: Vec<(u8, u8, u8)> = Vec::new();
    let mut exact = true;

    'outer: for y in 0..h {
        for x in 0..w {
            let c = frame.get_pixel(x, y);
            if !color_map.contains_key(&c) {
                if palette.len() >= 256 {
                    exact = false;
                    break 'outer;
                }
                let idx = palette.len() as u8;
                color_map.insert(c, idx);
                palette.push(c);
            }
        }
    }

    if exact {
        let indices = (0..h)
            .flat_map(|y| (0..w).map(move |x| (y, x)))
            .map(|(y, x)| color_map[&frame.get_pixel(x, y)])
            .collect();
        return (palette, indices);
    }

    // Fallback: 3-3-2 bit quantization → exactly 256 colors
    let mut pal = vec![(0u8, 0u8, 0u8); 256];
    for i in 0u32..256 {
        let r = ((i >> 5) * 255 / 7) as u8;
        let g = (((i >> 2) & 7) * 255 / 7) as u8;
        let b = ((i & 3) * 255 / 3) as u8;
        pal[i as usize] = (r, g, b);
    }

    let indices = (0..h)
        .flat_map(|y| (0..w).map(move |x| (y, x)))
        .map(|(y, x)| {
            let (r, g, b) = frame.get_pixel(x, y);
            ((r >> 5) << 5) | ((g >> 5) << 2) | (b >> 6)
        })
        .collect();

    (pal, indices)
}

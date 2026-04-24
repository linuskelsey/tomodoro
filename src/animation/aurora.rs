use super::*;

fn draw_snow_landscape(buf: &mut PixBuf, pw: usize, ph: usize) {
    let base_ground = ph * 84 / 100;
    let pi = std::f64::consts::PI;

    // Per-column ground height — gentle rolling hills
    let ground_at = |ppx: usize| -> usize {
        let xf = ppx as f64 / pw as f64;
        let h1 = ((xf * pi * 2.3).sin() * 0.5 + 0.5) * ph as f64 * 0.035;
        let h2 = ((xf * pi * 4.7 + 0.8).sin() * 0.5 + 0.5) * ph as f64 * 0.018;
        let noise = (hash(ppx as u64 * 17 + 333) % 4) as f64;
        ((base_ground as f64) - h1 - h2 - noise).max(0.0) as usize
    };

    // Helper: draw a mountain range with gradient base
    let draw_range = |buf: &mut PixBuf, base: usize, amp1: f64, freq1: f64, phase1: f64,
                                                       amp2: f64, freq2: f64, phase2: f64,
                      rock: (u8,u8,u8), snow_frac: f64| {
        for ppx in 0..pw {
            let xf  = ppx as f64 / pw as f64;
            let h1  = ((xf * pi * freq1 + phase1).sin() * 0.5 + 0.5) * ph as f64 * amp1;
            let h2  = ((xf * pi * freq2 + phase2).sin() * 0.5 + 0.5) * ph as f64 * amp2;
            let ht  = (h1 + h2) as usize;
            if ht == 0 { continue; }

            // Per-column noise on the base — breaks up the hard horizontal edge
            let base_noise = (hash(ppx as u64 * 13 + base as u64 * 7 + 444) % 7) as usize;
            let base_y  = (base + base_noise).min(ph);
            let top     = base_y.saturating_sub(ht);
            let snow_ln = top + (ht as f64 * snow_frac) as usize;
            let fade_h  = ((ht / 5) + 3).min(ht);
            let fade_y  = base_y.saturating_sub(fade_h);

            for py in top..base_y {
                if py >= buf.len() || ppx >= buf[0].len() { continue; }
                let noise = (hash(ppx as u64 * 5 + py as u64 * 9 + 1234) % 6) as f64 * 0.5;
                buf[py][ppx] = Some(if py < snow_ln {
                    let b = (py - top) as f64 / (snow_ln - top).max(1) as f64;
                    Color::Rgb((220.0 - b*25.0 + noise) as u8, (232.0 - b*22.0 + noise) as u8, (248.0 - b*14.0) as u8)
                } else if py < fade_y {
                    Color::Rgb(
                        (rock.0 as f64 + noise) as u8,
                        (rock.1 as f64 + noise) as u8,
                        (rock.2 as f64 + noise) as u8,
                    )
                } else {
                    // Gradient: rock → snow colour, with per-pixel noise for rough edge
                    let f = (py - fade_y) as f64 / fade_h as f64;
                    let n2 = (hash(ppx as u64 * 3 + py as u64 * 17 + 5678) % 10) as f64;
                    Color::Rgb(
                        (rock.0 as f64 + f * (190.0 - rock.0 as f64) + n2) as u8,
                        (rock.1 as f64 + f * (205.0 - rock.1 as f64) + n2) as u8,
                        (rock.2 as f64 + f * (222.0 - rock.2 as f64) + n2) as u8,
                    )
                });
            }
        }
    };

    // Distant range — more snow, lighter rock
    draw_range(buf, ph * 76 / 100, 0.14, 2.2, 0.0, 0.06, 4.3, 0.8, (22, 28, 48), 0.52);
    // Close range — more snow, heavier mountains
    draw_range(buf, base_ground,   0.24, 1.5, 0.5, 0.10, 3.1, 1.3, (8, 11, 20),  0.40);

    // Snow ground — per-column hill height
    for ppx in 0..pw {
        let gy = ground_at(ppx);
        for py in gy..ph {
            if py >= buf.len() || ppx >= buf[0].len() { continue; }
            let depth    = (py - gy) as f64 / (ph - gy).max(1) as f64;
            let aurora_t = (1.0 - depth) * 16.0;
            let v        = hash((ppx as u64 * 5).wrapping_add(py as u64 * 13).wrapping_add(9999));
            let noise    = (v % 6) as f64 * 0.5;
            let bright   = 182.0 + depth * 22.0;
            buf[py][ppx] = Some(Color::Rgb(
                (bright + noise) as u8,
                (bright + aurora_t + noise) as u8,
                (bright + aurora_t * 1.6 + noise + 10.0).min(255.0) as u8,
            ));
        }
    }
}

pub(super) fn fill_aurora(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let t = tick as f64 * 0.04;
    for py in 0..ph {
        for ppx in 0..pw {
            let frac = py as f64 / ph as f64;
            buf[py][ppx] = Some(Color::Rgb((3.0+frac*5.0) as u8, (5.0+frac*8.0) as u8, (15.0+frac*20.0) as u8));
        }
    }
    let aurora_h = ph * 7 / 10;
    let band_colors: [(u8,u8,u8); 4] = [(20,200,120),(40,180,220),(100,60,220),(20,220,160)];
    for ppx in 0..pw {
        let x = ppx as f64 / pw as f64;
        for band in 0..4u32 {
            let b = band as f64;
            let cx = (b * 0.27 + t * (0.3 + b * 0.1)).sin() * 0.5 + 0.5;
            let width = 0.12 + (t * 0.07 + b).sin() * 0.05;
            let dist = (x - cx).abs();
            if dist < width {
                let intensity = (1.0 - dist / width).powi(2);
                let band_h = (aurora_h as f64 * (0.4 + intensity * 0.5)) as usize;
                let bottom_f = aurora_h as f64
                    + (b * 1.7 + t * 0.05).sin()               * ph as f64 * 0.10
                    + (ppx as f64 * 0.03 + t * 0.18 + b * 2.1).sin() * ph as f64 * 0.07
                    + (ppx as f64 * 0.09 + t * 0.31 + b * 0.9).sin() * ph as f64 * 0.04
                    + (ppx as f64 * 0.23 + t * 0.14 + b * 3.3).sin() * ph as f64 * 0.02;
                let bottom = (bottom_f as isize).clamp(0, ph as isize) as usize;
                let top = bottom.saturating_sub(band_h);
                let (r, g, bl) = band_colors[band as usize % band_colors.len()];
                for py in top..bottom.min(ph) {
                    let depth = if band_h > 0 { (py - top) as f64 / band_h as f64 } else { 0.0 };
                    let alpha = intensity * (1.0 - depth * 0.7);
                    let existing = buf[py][ppx];
                    let (er, eg, eb) = match existing {
                        Some(Color::Rgb(a, b, c)) => (a as f64, b as f64, c as f64),
                        _ => (0.0, 0.0, 0.0),
                    };
                    buf[py][ppx] = Some(Color::Rgb(
                        (er + r as f64 * alpha).min(255.0) as u8,
                        (eg + g as f64 * alpha).min(255.0) as u8,
                        (eb + bl as f64 * alpha).min(255.0) as u8,
                    ));
                }
            }
        }
    }
    for i in 0..(pw * ph / 60).max(5) {
        let seed = hash(i as u64 + 5555);
        let sx = (seed % pw as u64) as usize;
        let sy = (seed >> 8) as usize % (ph / 2).max(1);
        let tw = ((tick as f64 * 0.1 + i as f64 * 1.3).sin() * 0.4 + 0.6) * 160.0;
        let v = tw as u8;
        if let Some(Color::Rgb(er, eg, eb)) = buf[sy][sx] {
            if (er as u16 + eg as u16 + eb as u16) < 150 {
                buf[sy][sx] = Some(Color::Rgb(v, v, v));
            }
        }
    }
    draw_snow_landscape(buf, pw, ph);
}

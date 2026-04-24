use super::*;

fn draw_ufo(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let period  = 380u64;
    let visible = 100u64;
    let phase   = tick % period;
    if phase >= visible { return; }

    let crossing  = tick / period;
    let progress  = phase as f64 / visible as f64;
    // alternate direction each crossing
    let cx = if crossing % 2 == 0 {
        (progress * (pw as f64 + 24.0)) as isize - 12
    } else {
        (pw as f64 + 12.0 - progress * (pw as f64 + 24.0)) as isize
    };
    // vary y position between crossings
    let base_y  = (ph as f64 * (0.30 + (hash(crossing * 31 + 7) % 20) as f64 * 0.01)) as isize;
    let bob     = ((tick as f64 * 0.18).sin() * 2.5) as isize;
    let cy      = base_y + bob;

    let body_hi = Color::Rgb(168, 170, 182);
    let body_dk = Color::Rgb(108, 110, 124);
    let dome_hi = Color::Rgb(85, 195, 230);
    let dome_dk = Color::Rgb(48, 142, 178);
    let rim_col = Color::Rgb(200, 200, 210);

    // Saucer body — wide ellipse 2px tall
    for dx in -5isize..=5 {
        let inner = dx.abs() <= 2;
        set_px(buf, cx + dx, cy,     if inner { body_hi } else { body_dk });
        set_px(buf, cx + dx, cy + 1, body_dk);
    }
    // Rim highlight
    set_px(buf, cx - 5, cy, rim_col);
    set_px(buf, cx + 5, cy, rim_col);

    // Dome
    for dx in -2isize..=2 { set_px(buf, cx + dx, cy - 1, dome_dk); }
    for dx in -1isize..=1 { set_px(buf, cx + dx, cy - 2, dome_hi); }
    set_px(buf, cx, cy - 3, dome_dk);

    // Dim pulsing lights on underside
    let pulse = (tick as f64 * 0.12).sin() * 0.5 + 0.5;
    let v = (140.0 + pulse * 50.0) as u8;
    for lx in [-3isize, 0, 3] {
        set_px(buf, cx + lx, cy + 2, Color::Rgb(v, v, v));
    }

    // Tractor beam — fades in at start of crossing
    if phase < 35 {
        let alpha = 1.0 - phase as f64 / 35.0;
        let beam_h = (ph as f64 * 0.18 * alpha) as isize;
        for dy in 3..3 + beam_h {
            let spread = (dy / 5).min(3);
            for dx in -spread..=spread {
                let v = hash((cx + dx).unsigned_abs() as u64 * 7 + (cy + dy).unsigned_abs() as u64 * 13 + tick);
                if v % 3 != 0 {
                    set_px(buf, cx + dx, cy + dy, Color::Rgb(
                        (180.0 * alpha) as u8,
                        (210.0 * alpha) as u8,
                        (255.0 * alpha) as u8,
                    ));
                }
            }
        }
    }
}

pub(super) fn fill_stars(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    for py in 0..ph { for ppx in 0..pw { buf[py][ppx] = Some(Color::Rgb(0, 0, 8)); } }
    let layers: &[(usize, u8, u8, bool)] = &[
        (1, 90,  110, false),
        (2, 170, 190, false),
        (4, 240, 255, true),
    ];
    let n = (pw * ph / 80).max(8);
    for (depth, (speed, dim, bright, trail)) in layers.iter().enumerate() {
        for i in 0..n {
            let seed = hash(i as u64 * 3 + depth as u64 * 997);
            let y  = (seed % ph as u64) as usize;
            let x0 = (seed >> 10) as usize % pw;
            let x  = (x0 + pw - (tick as usize * speed / 10) % pw) % pw;
            let c  = Color::Rgb(*bright, *bright, *bright);
            buf[y][x] = Some(c);
            if *trail {
                for t in 1..4usize {
                    let tx = (x + t) % pw;
                    let fade = dim.saturating_sub(t as u8 * 25);
                    buf[y][tx] = Some(Color::Rgb(fade, fade, fade + 15));
                }
            }
        }
    }
    for i in 0..(pw * ph / 400).max(2) {
        let seed = hash(i as u64 * 7919 + 42);
        let bx = (seed % pw as u64) as usize;
        let by = (seed >> 10) as usize % ph;
        let tw = ((tick as f64 * 0.07 + i as f64).sin() * 0.5 + 0.5) * 255.0;
        let v  = tw as u8;
        let c  = Color::Rgb(v, v, (v as u16 + 40).min(255) as u8);
        buf[by][bx] = Some(c);
        if bx + 1 < pw { buf[by][bx+1] = Some(c); }
        if by + 1 < ph { buf[by+1][bx] = Some(c); }
    }
    draw_ufo(buf, pw, ph, tick);
}

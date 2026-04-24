use super::*;

fn draw_boat(buf: &mut PixBuf, pw: usize, ph: usize, t: f64) {
    let horizon    = ph * 3 / 10;
    let bcx        = pw / 3;
    // Boat rides waves but clamped well below horizon into the water body
    let w1         = (bcx as f64 * 0.35 + t).sin() * 2.5;
    let w2         = (bcx as f64 * 0.18 - t * 0.65).sin() * 1.5;
    let water_surf = horizon as f64 + w1 + w2;
    let water_y    = (water_surf + ph as f64 * 0.15).max(horizon as f64 + ph as f64 * 0.12);
    // Slope for slight tilt
    let w1r        = ((bcx + 4) as f64 * 0.35 + t).sin() * 2.5;
    let w2r        = ((bcx + 4) as f64 * 0.18 - t * 0.65).sin() * 1.5;
    let slope      = ((horizon as f64 + w1r + w2r) - water_surf) / 4.0;

    let half_deck  = (pw / 12).max(3) as isize;
    let hull_h     = (ph / 12).max(2) as isize;
    let hull_dark  = Color::Rgb(55, 35, 16);
    let hull_mid   = Color::Rgb(78, 52, 24);
    let deck_col   = Color::Rgb(105, 72, 36);
    let mast_col   = Color::Rgb(70, 46, 20);
    let sail_col   = Color::Rgb(218, 210, 192);

    // Hull rows: trapezoid, wide at deck, narrow at keel
    let half_keel  = half_deck / 2;
    for row in 0..hull_h {
        let frac     = row as f64 / hull_h as f64;
        let hw       = (half_deck as f64 - frac * (half_deck - half_keel) as f64) as isize;
        let tilt_off = (slope * (row as f64 - hull_h as f64 / 2.0)) as isize;
        let deck_y   = water_y as isize - hull_h + row;
        for dx in -hw..=hw {
            let c = if row == 0 { deck_col }
                    else if dx == -hw || dx == hw { hull_dark }
                    else { hull_mid };
            set_px(buf, bcx as isize + dx + tilt_off, deck_y, c);
        }
    }

    // Mast
    let mast_base = water_y as isize - hull_h;
    let mast_h    = hull_h * 3;
    for dy in 0..mast_h { set_px(buf, bcx as isize, mast_base - dy, mast_col); }

    // Triangular sail
    let sail_h = mast_h * 2 / 3;
    for dy in 0..sail_h {
        let sail_w = (half_deck as f64 * (sail_h - dy) as f64 / sail_h as f64) as isize;
        for dx in 1..sail_w { set_px(buf, bcx as isize + dx, mast_base - dy, sail_col); }
    }
}

pub(super) fn fill_waves(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let t = tick as f64 * 0.12;
    let horizon = ph * 3 / 10;
    for py in 0..ph {
        for ppx in 0..pw {
            buf[py][ppx] = Some(if py < horizon {
                let frac = py as f64 / horizon as f64;
                Color::Rgb((8.0 + frac*18.0) as u8, (18.0 + frac*38.0) as u8, (55.0 + frac*75.0) as u8)
            } else {
                let wy  = py - horizon;
                let wh  = ph - horizon;
                let wyf = wy as f64;
                let whf = wh as f64;
                let depth = wyf / whf;

                // Wave trains — surface and three sub-surface layers
                let w1 = (ppx as f64 * 0.35 + t).sin() * 2.5;
                let w2 = (ppx as f64 * 0.18 - t * 0.65).sin() * 1.5;
                let surf = (wyf - (w1 + w2)).abs();

                let w3   = (ppx as f64 * 0.52 + t * 1.2).sin() * 1.8;
                let w4   = (ppx as f64 * 0.27 - t * 0.85).sin() * 1.3;
                let d1   = whf * 0.11;
                let surf2 = (wyf - d1 - (w3 + w4)).abs();

                let w5   = (ppx as f64 * 0.66 - t * 1.45).sin() * 1.4;
                let w6   = (ppx as f64 * 0.15 + t * 0.42).sin() * 1.9;
                let d2   = whf * 0.23;
                let surf3 = (wyf - d2 - (w5 + w6 * 0.6)).abs();

                let w7   = (ppx as f64 * 0.80 + t * 1.75).sin() * 1.1;
                let w8   = (ppx as f64 * 0.38 - t * 0.55).sin() * 1.6;
                let d3   = whf * 0.38;
                let surf4 = (wyf - d3 - (w7 + w8 * 0.7)).abs();

                // Cross-chop — short diagonal ripples across the body
                let chop1 = (ppx as f64 * 0.9 + wyf * 0.4 + t * 2.1).sin();
                let chop2 = (ppx as f64 * 0.7 - wyf * 0.35 - t * 1.8).sin();
                let chop  = (chop1 * chop2).abs();

                // Subtle specular glints
                let refl  = (ppx as f64 * 0.13 - t * 0.28).sin()
                           * (ppx as f64 * 0.08 + t * 0.19).sin();

                let base_g = (90.0 - depth * 50.0).max(0.0);
                let base_b = (200.0 - depth * 80.0).max(0.0);

                if surf < 1.3 {
                    Color::Rgb(205, 232, 255)
                } else if surf < 2.6 {
                    Color::Rgb(125, 188, 232)
                } else if surf2 < 1.0 {
                    Color::Rgb(80, 158, 218)
                } else if surf2 < 2.0 {
                    Color::Rgb(30, 122, 200)
                } else if surf3 < 0.9 {
                    Color::Rgb(18, 108, 185)
                } else if surf3 < 1.8 {
                    Color::Rgb(8, 95, 172)
                } else if surf4 < 0.85 && depth < 0.72 {
                    Color::Rgb(12, 102, 168)
                } else if chop > 0.82 && depth < 0.55 {
                    // cross-chop glitter
                    Color::Rgb((12.0 + chop * 28.0) as u8, (base_g + chop * 22.0).min(255.0) as u8, (base_b + chop * 14.0).min(255.0) as u8)
                } else if refl > 0.68 && depth < 0.42 {
                    // specular glint near surface
                    Color::Rgb((18.0 + refl * 45.0) as u8, (base_g + refl * 32.0).min(255.0) as u8, (base_b + refl * 18.0).min(255.0) as u8)
                } else {
                    Color::Rgb(0, base_g as u8, base_b as u8)
                }
            });
        }
    }
    let mx = pw * 4 / 5;
    let my = ph / 8;
    let mr = (pw.min(ph) / 10).max(2) as f64;
    for py in 0..ph {
        for ppx in 0..pw {
            let dx = ppx as f64 - mx as f64;
            let dy = (py as f64 - my as f64) * 1.6;
            if dx*dx + dy*dy < mr*mr { buf[py][ppx] = Some(Color::Rgb(255, 240, 180)); }
        }
    }
    draw_boat(buf, pw, ph, t);
}

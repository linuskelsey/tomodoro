use super::*;

pub(super) fn fill_sunset(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let t   = tick as f64 * 0.07;
    let hor = ph * 46 / 100;

    // Sky: deep violet at top → warm orange-gold at horizon
    let sky_rgb = |py: usize| -> (u8, u8, u8) {
        let f = (py as f64 / hor as f64).min(1.0);
        if f < 0.38 {
            let g = f / 0.38;
            ((22.0 + g * 90.0) as u8, (8.0 + g * 14.0) as u8, (52.0 + g * 8.0) as u8)
        } else if f < 0.72 {
            let g = (f - 0.38) / 0.34;
            ((112.0 + g * 95.0) as u8, (22.0 + g * 38.0) as u8, (60.0 - g * 32.0).max(0.0) as u8)
        } else {
            let g = (f - 0.72) / 0.28;
            ((207.0 + g * 48.0) as u8, (60.0 + g * 80.0) as u8, (28.0 - g * 8.0).max(0.0) as u8)
        }
    };

    for py in 0..hor.min(ph) {
        let (r, g, b) = sky_rgb(py);
        for ppx in 0..pw { buf[py][ppx] = Some(Color::Rgb(r, g, b)); }
    }

    // Faint stars visible in upper purple sky
    for i in 0..(pw * ph / 120).max(4) {
        let seed = hash(i as u64 + 7777);
        let sx = (seed % pw as u64) as usize;
        let sy = (seed >> 10) as usize % (hor * 35 / 100).max(1);
        let tw = ((tick as f64 * 0.06 + i as f64 * 1.7).sin() * 0.35 + 0.65) * 155.0;
        let v  = tw as u8;
        buf[sy][sx] = Some(Color::Rgb(v, v, (v as u16 + 25).min(255) as u8));
    }

    // Sun glow
    let sun_cx = (pw * 55 / 100) as isize;
    let sun_cy = hor.saturating_sub(ph * 3 / 100) as isize;
    let sun_r  = (ph / 20).max(2) as f64;
    for dy in -(sun_r as isize * 4)..=(sun_r as isize * 4) {
        for dx in -(sun_r as isize * 4)..=(sun_r as isize * 4) {
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            let py   = sun_cy + dy;
            let ppx  = sun_cx + dx;
            if py < 0 || py >= hor as isize || ppx < 0 || ppx as usize >= pw { continue; }
            if dist <= sun_r {
                buf[py as usize][ppx as usize] = Some(Color::Rgb(255, 248, 195));
            } else if dist <= sun_r * 3.5 {
                let gf = (1.0 - (dist - sun_r) / (sun_r * 2.5)).max(0.0);
                if let Some(Color::Rgb(r, g, b)) = buf[py as usize][ppx as usize] {
                    buf[py as usize][ppx as usize] = Some(Color::Rgb(
                        (r as f64 + 88.0 * gf).min(255.0) as u8,
                        (g as f64 + 55.0 * gf).min(255.0) as u8,
                        (b as f64 + 15.0 * gf).min(255.0) as u8,
                    ));
                }
            }
        }
    }

    // Mountain silhouette
    let mtn_ht = |ppx: usize| -> usize {
        let xf = ppx as f64 / pw as f64;
        let h1 = ((xf * std::f64::consts::PI * 3.4).sin() * 0.5 + 0.5) * ph as f64 * 0.20;
        let h2 = ((xf * std::f64::consts::PI * 6.8 + 1.4).sin() * 0.5 + 0.5) * ph as f64 * 0.10;
        let h3 = (hash(ppx as u64 * 17 + 9191) % (ph as u64 / 18 + 1)) as f64;
        (h1 + h2 + h3) as usize
    };
    for ppx in 0..pw {
        let top = hor.saturating_sub(mtn_ht(ppx));
        for py in top..hor.min(ph) { buf[py][ppx] = Some(Color::Rgb(18, 8, 32)); }
    }

    // Shore strip at horizon
    for py in hor..(hor + 2).min(ph) {
        for ppx in 0..pw { buf[py][ppx] = Some(Color::Rgb(10, 5, 20)); }
    }

    let lake_top   = (hor + 2).min(ph);
    let grass_base = ph * 78 / 100;

    // Per-column undulating grass top — natural hillside edge
    let grass_col_top = |ppx: usize| -> usize {
        let xf = ppx as f64 / pw as f64;
        let w1  = (xf * std::f64::consts::PI * 2.3).sin() * ph as f64 * 0.028;
        let w2  = (xf * std::f64::consts::PI * 5.7 + 1.1).sin() * ph as f64 * 0.014;
        let nz  = (hash(ppx as u64 * 23 + 4433) % (ph as u64 / 22 + 1)) as f64 - ph as f64 / 44.0;
        (grass_base as f64 - w1 - w2 - nz)
            .clamp(lake_top as f64 + 1.0, ph as f64 - 2.0) as usize
    };

    let max_lake_y = (grass_base + ph / 10).min(ph);

    // Lake and lower grass in one pass (lake where py < grass edge, grass otherwise)
    for py in lake_top..max_lake_y {
        let depth_lake = (py - lake_top) as f64 / (ph - lake_top).max(1) as f64;
        let dark = 1.0 - depth_lake * 0.38;
        for ppx in 0..pw {
            let gct = grass_col_top(ppx);
            if py >= gct {
                let depth = (py - gct) as f64 / (ph - gct).max(1) as f64;
                let v = hash(ppx as u64 * 5 + py as u64 * 9 + 5566);
                let n = (v % 10) as f64;
                buf[py][ppx] = Some(Color::Rgb(
                    (20.0 + depth * 8.0 + n * 0.3) as u8,
                    (45.0 + depth * 14.0 + n * 0.9) as u8,
                    (12.0 + depth * 5.0  + n * 0.2) as u8,
                ));
            } else {
                let ripple = ((ppx as f64 * 0.28 + t * 1.6).sin() * 1.4
                            + (ppx as f64 * 0.65 - t * 0.9).sin() * 0.7) as isize;
                let md  = py - lake_top;
                let spy = (hor.saturating_sub(md + 2) as isize + ripple).clamp(0, hor as isize - 1) as usize;
                let (r, g, b) = sky_rgb(spy);
                buf[py][ppx] = Some(Color::Rgb(
                    (r as f64 * dark) as u8,
                    (g as f64 * dark) as u8,
                    (b as f64 * dark) as u8,
                ));
            }
        }
    }

    // Remaining grass rows below the transition zone
    for py in max_lake_y..ph {
        let depth = (py - grass_base) as f64 / (ph - grass_base).max(1) as f64;
        for ppx in 0..pw {
            let v = hash(ppx as u64 * 5 + py as u64 * 9 + 5566);
            let n = (v % 10) as f64;
            buf[py][ppx] = Some(Color::Rgb(
                (20.0 + depth * 8.0 + n * 0.3) as u8,
                (45.0 + depth * 14.0 + n * 0.9) as u8,
                (12.0 + depth * 5.0  + n * 0.2) as u8,
            ));
        }
    }

    // Mountain reflection in lake
    for ppx in 0..pw {
        let gct = grass_col_top(ppx);
        let ht  = mtn_ht(ppx);
        for di in 0..ht.min(gct.saturating_sub(lake_top)) {
            let py = lake_top + di;
            if py >= ph { break; }
            let df  = di as f64 / ht.max(1) as f64;
            let drk = 1.0 - df * 0.45;
            let rx  = (ppx as isize + ((ppx as f64 * 0.45 + t * 2.2).sin() as isize))
                      .clamp(0, pw as isize - 1) as usize;
            let base = (18.0 * drk) as u8;
            buf[py][rx] = Some(Color::Rgb(base, (base as f64 * 0.44) as u8, (base as f64 * 1.78).min(255.0) as u8));
        }
    }

    // Specular shimmer on lake
    let n_glints = (pw / 3).max(4);
    for i in 0..n_glints {
        let h = hash(i as u64 * 13 + tick / 8 + 8888);
        if h % 3 != 0 { continue; }
        let gx  = (h >> 5) as usize % pw;
        let gy  = lake_top + (h >> 15) as usize % ((grass_base.saturating_sub(lake_top)) / 3).max(1);
        let gct = grass_col_top(gx);
        if gy >= gct { continue; }
        let gv = (145.0 + ((tick as f64 * 0.22 + i as f64).sin() * 55.0)) as u8;
        set_px(buf, gx as isize, gy as isize, Color::Rgb(gv, (gv as f64 * 0.68) as u8, (gv as f64 * 0.36) as u8));
    }

    // Grass blade tips along undulating edge
    for ppx in 0..pw {
        let h = hash(ppx as u64 * 13 + 9988);
        if h % 4 != 0 { continue; }
        let blade_h = 2 + (h >> 12) as usize % 3;
        let gct  = grass_col_top(ppx);
        let sway = ((ppx as f64 * 0.12 + tick as f64 * 0.04).sin() * 1.5) as isize;
        for dy in 0..blade_h.min(ph.saturating_sub(gct)) {
            let tf = dy as f64 / blade_h.max(1) as f64;
            set_px(buf, ppx as isize + (sway as f64 * tf) as isize,
                   (gct + dy) as isize, Color::Rgb(35, 68, 20));
        }
    }
}

use super::*;

fn trunk_half_w(t: f64, pw: usize) -> isize {
    // Uniform trunk — very slight taper, mostly cylindrical
    ((0.11 + t * 0.04) * pw as f64) as isize
}

fn draw_autumn_cluster(buf: &mut PixBuf, cx: isize, cy: isize, r: isize, seed: u64, pal: usize) {
    if r <= 0 { return; }
    let palettes: &[[Color; 3]] = &[
        [Color::Rgb(180, 48, 8),  Color::Rgb(200, 70, 12),  Color::Rgb(155, 35, 5) ],
        [Color::Rgb(195, 120, 0), Color::Rgb(215, 148, 8),  Color::Rgb(170, 98,  0)],
        [Color::Rgb(162, 72, 15), Color::Rgb(185, 92,  20), Color::Rgb(140, 55, 10)],
    ];
    let cols = &palettes[pal % palettes.len()];
    for dy in -r..=r {
        for dx in -(r * 3/2)..=(r * 3/2) {
            let ex = dx * 2 / 3;
            if ex * ex + dy * dy <= r * r {
                let v = hash(cx.unsigned_abs() as u64 * 7 + cy.unsigned_abs() as u64 * 13
                          + dx.unsigned_abs() as u64 * 3 + dy.unsigned_abs() as u64 + seed);
                if v % 8 > 1 { set_px(buf, cx + dx, cy + dy, cols[(v % 3) as usize]); }
            }
        }
    }
}

fn draw_autumn_tree_b(buf: &mut PixBuf, ph: usize, tx: isize, base_y: usize, scale: f64, seed: u64, pal: usize) {
    let trk   = Color::Rgb(32, 18, 7);
    let trk_d = Color::Rgb(20, 10, 3);
    let th    = (scale * ph as f64 * 0.115).max(5.0) as isize;
    let tw    = (scale * 0.75).max(1.0) as isize;
    let bl    = (scale * ph as f64 * 0.09).max(4.0) as isize;
    for dy in 0..th {
        let jit = ((hash(seed + dy as u64 * 7) % 3) as isize - 1) / 2;
        let w   = (tw as f64 * (1.0 - dy as f64 / th as f64 * 0.15)).max(1.0) as isize;
        for dx in -w..=w {
            set_px(buf, tx + dx + jit, base_y as isize - dy,
                   if dx.abs() == w { trk_d } else { trk });
        }
    }
    let f1 = base_y as isize - th * 62 / 100;
    let f2 = base_y as isize - th;
    let brs: &[(isize,isize,isize,isize,u64,bool)] = &[
        (f1, -bl,        -(bl*6/10), bl,       100, true),
        (f1,  bl*9/10,   -(bl*6/10), bl*9/10,  200, true),
        (f2, -(bl*7/10), -bl,        bl*7/10,  300, false),
        (f2,  bl*4/5,    -bl,        bl*4/5,   400, false),
        (f2,  0,         -(bl/2),    bl/2,     500, false),
    ];
    for &(by, edx, edy, bln, so, has_sub) in brs {
        if bln == 0 { continue; }
        for i in 0..bln {
            let px = tx + edx * i / bln;
            let py = by + edy * i / bln;
            let bw = if i < bln / 4 { 1isize } else { 0 };
            for w in -bw..=bw {
                set_px(buf, px + w, py, if bw > 0 && w.abs() == bw { trk_d } else { trk });
            }
        }
        let cr = (bln * 11 / 20).max(3);
        draw_autumn_cluster(buf, tx + edx, by + edy, cr, seed + so, pal);
        if !has_sub { continue; }
        let mx = tx + edx / 2; let my = by + edy / 2; let sl = bln * 11 / 20;
        if sl < 2 { continue; }
        for &sd in &[-1isize, 1] {
            for i in 0..sl {
                set_px(buf, mx + sd * bl / 4 * i / sl, my - bl * 2 / 5 * i / sl, trk_d);
            }
            let scr = (sl * 2 / 5).max(2);
            draw_autumn_cluster(buf, mx + sd * bl / 4, my - bl * 2 / 5, scr, seed + so + 600, pal);
        }
    }
}

fn draw_background_trees(buf: &mut PixBuf, pw: usize, ph: usize) {
    let ground_y = ph * 85 / 100;

    // (x_frac, size_frac, palette)
    let trees: &[(f64, f64, usize)] = &[
        (0.07, 0.52, 0),
        (0.17, 0.44, 1),
        (0.27, 0.62, 2),
        (0.73, 0.48, 2),
        (0.83, 0.58, 1),
        (0.93, 0.40, 0),
    ];

    for (i, &(xf, sf, pal)) in trees.iter().enumerate() {
        let tx    = (xf * pw as f64) as isize;
        let scale = sf / 0.115;
        draw_autumn_tree_b(buf, ph, tx, ground_y, scale, (i as u64 + 1) * 20000, pal);
    }
}

fn draw_moss(buf: &mut PixBuf, pw: usize, ph: usize) {
    let ground_y = ph * 85 / 100;
    let cx       = pw as isize / 2;
    let moss_dk  = Color::Rgb(20, 42, 16);
    let moss_mid = Color::Rgb(30, 58, 23);
    let moss_lt  = Color::Rgb(42, 75, 32);

    for py in ground_y..ph {
        let t      = py as f64 / ph as f64;
        let hw     = trunk_half_w(t, pw) as f64;
        for ppx in 0..pw {
            let dist   = (ppx as isize - cx).abs() as f64;
            let beyond = dist - hw;
            if beyond > 0.0 && beyond < pw as f64 * 0.28 {
                let proximity = 1.0 - (beyond / (pw as f64 * 0.28)).min(1.0);
                let v = hash((ppx as u64 * 7).wrapping_add(py as u64 * 11).wrapping_add(888));
                if ((v % 10) as f64) < 2.5 + proximity * 5.5 {
                    buf[py][ppx] = Some(match v % 3 {
                        0 => moss_dk,
                        1 => moss_mid,
                        _ => moss_lt,
                    });
                }
            }
        }
    }
}

fn draw_maple_tree(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let cx = pw as isize / 2;

    let very_dark = Color::Rgb(25, 13, 5);
    let dark      = Color::Rgb(42, 24, 9);
    let mid       = Color::Rgb(58, 34, 14);
    let light     = Color::Rgb(74, 46, 20);

    for py in 0..ph {
        let t     = py as f64 / ph as f64;
        let hw    = trunk_half_w(t, pw);
        let lj    = (hash(py as u64 * 7 + 13) % 3) as isize - 1;
        let rj    = (hash(py as u64 * 11 + 29) % 3) as isize - 1;
        let left  = cx - hw + lj;
        let right = cx + hw + rj;
        for ppx in left..right {
            let edge_d = (ppx - left).min(right - ppx);
            let stripe = hash((ppx as u64).wrapping_mul(3).wrapping_add(py as u64 / 4 * 7) + 42) % 10;
            let c = if edge_d <= 1 { very_dark }
                    else if stripe == 0 { very_dark }
                    else if stripe <= 3 { dark }
                    else if stripe <= 6 { mid }
                    else { light };
            set_px(buf, ppx, py as isize, c);
        }
    }

    // Knots
    for k in 0..3usize {
        let ky = (ph * (k + 1) / 4) as isize;
        let kx = cx + [-4isize, 5, -3][k];
        fill_r(buf, kx - 2, ky - 1, 5, 3, very_dark);
        set_px(buf, kx, ky, Color::Rgb(15, 8, 3));
    }

    // Shimenawa
    let rope_y   = (ph * 30 / 100) as isize;
    let rope_hw  = trunk_half_w(0.30, pw) + 1;
    let rope_l   = cx - rope_hw;
    let rope_r   = cx + rope_hw;
    let rope_col = Color::Rgb(185, 155, 80);
    let rope_dk  = Color::Rgb(140, 115, 55);
    for ppx in rope_l..rope_r {
        let alt = (ppx % 3 == 0) as isize;
        set_px(buf, ppx, rope_y + alt,     rope_col);
        set_px(buf, ppx, rope_y + 2 + alt, rope_dk);
    }

    // Shide
    let shide_col = Color::Rgb(235, 230, 218);
    let shide_dk  = Color::Rgb(195, 190, 178);
    let shide_h   = (ph / 10).max(5);
    for i in 0..7usize {
        let sx     = rope_l + (rope_r - rope_l) * i as isize / 6;
        let sway   = ((tick as f64 * 0.015 + i as f64 * 1.1).sin() * 1.2) as isize;
        let base_x = sx + sway;
        for dy in 0..shide_h as isize {
            let ox = if (dy * 3 / shide_h as isize) % 2 == 0 { 0isize } else { 1 };
            let c  = if dy % 2 == 0 { shide_col } else { shide_dk };
            set_px(buf, base_x + ox,     rope_y + 4 + dy, c);
            set_px(buf, base_x + ox + 1, rope_y + 4 + dy, c);
        }
    }
}

pub(super) fn fill_leaves(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let ground_y = ph * 85 / 100;
    for py in 0..ph {
        for ppx in 0..pw {
            buf[py][ppx] = Some(if py < ground_y {
                let f = py as f64 / ground_y as f64;
                Color::Rgb((35.0 + f*12.0) as u8, (28.0 + f*10.0) as u8, (20.0 + f*6.0) as u8)
            } else {
                let f = (py - ground_y) as f64 / (ph - ground_y) as f64;
                Color::Rgb((45.0 + f*15.0) as u8, (30.0 + f*10.0) as u8, (15.0 + f*5.0) as u8)
            });
        }
    }
    draw_background_trees(buf, pw, ph);
    draw_moss(buf, pw, ph);
    draw_maple_tree(buf, pw, ph, tick);

    let leaf_colors = [
        Color::Rgb(210, 65, 10), Color::Rgb(195, 130, 0),
        Color::Rgb(170, 75, 20), Color::Rgb(145, 95, 30), Color::Rgb(220, 160, 0),
    ];
    // Falling leaves — full height, no ground_y cap
    let n_leaves = (pw * ph / 40).max(12).min(80);
    for i in 0..n_leaves {
        let h1 = hash(i as u64 + 1);
        let h2 = hash(i as u64 + 1000);
        let x_base = (h1 % pw as u64) as usize;
        let y_start = (h2 % ph as u64) as usize;
        let speed = 1 + (h1 >> 20) % 2;
        let sway_a = 3.0 + (h2 >> 10 & 0xf) as f64 * 0.3;
        let sway_f = 0.04 + (h1 >> 15 & 0x7) as f64 * 0.005;
        let phase_s = (h2 >> 8 & 0x3f) as f64;
        let x = (x_base as f64 + (tick as f64 * sway_f + phase_s).sin() * sway_a) as isize;
        let y = ((y_start + tick as usize * speed as usize) % ph) as isize;
        let col = leaf_colors[(h1 as usize) % leaf_colors.len()];
        for dy in 0..2isize {
            for dx in 0..2isize {
                set_px(buf, x + dx, y + dy, col);
            }
        }
    }

    // Scattered leaves on floor — static positions, slightly faded
    let n_floor = (pw * (ph - ground_y) / 8).max(6).min(40);
    for i in 0..n_floor {
        let h = hash(i as u64 * 3 + 7777);
        let fx = (h % pw as u64) as isize;
        let fy = (ground_y + (h >> 10) as usize % (ph - ground_y)) as isize;
        let col = leaf_colors[(h as usize) % leaf_colors.len()];
        // Single pixel leaves on floor — no alpha fade needed, moss/ground visible around them
        set_px(buf, fx, fy, col);
        if (h >> 20) % 3 != 0 { set_px(buf, fx + 1, fy, col); }
    }
}

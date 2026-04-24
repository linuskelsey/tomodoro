use super::*;

fn draw_blossom_cluster(buf: &mut PixBuf, cx: isize, cy: isize, r: isize, seed: u64) {
    if r <= 0 { return; }
    let cols = [Color::Rgb(255,215,228), Color::Rgb(245,178,202), Color::Rgb(222,142,168)];
    for dy in -r..=r {
        for dx in -(r * 3/2)..=(r * 3/2) {
            let ex = dx * 2 / 3;
            if ex * ex + dy * dy <= r * r {
                let v = hash(cx.unsigned_abs() as u64 * 7 + cy.unsigned_abs() as u64 * 13
                          + dx.unsigned_abs() as u64 * 3 + dy.unsigned_abs() as u64 + seed);
                if v % 5 > 0 { set_px(buf, cx + dx, cy + dy, cols[(v % 3) as usize]); }
            }
        }
    }
}

fn draw_cherry_tree_b(buf: &mut PixBuf, ph: usize, tx: isize, base_y: usize, scale: f64, seed: u64) {
    let trk   = Color::Rgb(65, 38, 20);
    let trk_d = Color::Rgb(44, 25, 10);
    let th    = (scale * ph as f64 * 0.115).max(5.0) as isize;
    let tw    = (scale * 1.5).max(1.0) as isize;
    let bl    = (scale * ph as f64 * 0.09).max(4.0) as isize;
    for dy in 0..th {
        let jit = ((hash(seed + dy as u64 * 7) % 3) as isize - 1) / 2;
        let w   = (tw as f64 * (1.0 - dy as f64 / th as f64 * 0.45)).max(1.0) as isize;
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
        let cr = (bln * 7 / 20).max(2);
        draw_blossom_cluster(buf, tx + edx, by + edy, cr, seed + so);
        if !has_sub { continue; }
        let mx = tx + edx / 2; let my = by + edy / 2; let sl = bln * 11 / 20;
        if sl < 2 { continue; }
        for &sd in &[-1isize, 1] {
            for i in 0..sl {
                set_px(buf, mx + sd * bl / 4 * i / sl, my - bl * 2 / 5 * i / sl, trk_d);
            }
            let scr = (sl * 1 / 4).max(1);
            draw_blossom_cluster(buf, mx + sd * bl / 4, my - bl * 2 / 5, scr, seed + so + 600);
        }
    }
}

fn draw_tea_setup(buf: &mut PixBuf, pw: usize, ph: usize) {
    let cx    = (pw * 62 / 100) as isize;
    let ty    = (ph * 91 / 100) as isize;
    let tw    = (pw / 8).max(8) as isize;
    let tleg  = (ph / 14).max(3) as isize;
    let wood  = Color::Rgb(92, 58, 22);
    let wood_d = Color::Rgb(62, 38, 12);
    for dx in -tw..=tw {
        set_px(buf, cx + dx, ty, wood);
        set_px(buf, cx + dx, ty + 1, wood_d);
    }
    for &lx in &[cx - tw + tw/4, cx + tw - tw/4] {
        for dy in 1..=tleg { set_px(buf, lx, ty + dy, wood_d); }
    }
    let pot_r = (pw as isize / 22).max(2);
    let px = cx - tw / 4;
    for dy in -pot_r..=pot_r {
        let hw = (pot_r as f64 * (1.0 - (dy as f64 / (pot_r as f64 + 0.5)).powi(2)).sqrt()).max(0.0) as isize;
        for dx in -hw..=hw {
            set_px(buf, px + dx, ty - 1 + dy,
                   if dy == -pot_r || dx.abs() == hw { Color::Rgb(42,42,42) } else { Color::Rgb(25,25,25) });
        }
    }
    set_px(buf, px, ty - 1 - pot_r - 1, Color::Rgb(55,55,55));
    for dx in -1isize..=1 { set_px(buf, px + dx, ty - 1 - pot_r, Color::Rgb(42,42,42)); }
    set_px(buf, px + pot_r + 1, ty - 1 - pot_r/2, Color::Rgb(42,42,42));
    set_px(buf, px + pot_r + 2, ty - 2 - pot_r/2, Color::Rgb(42,42,42));
    set_px(buf, px - pot_r - 1, ty - 1 - pot_r/2, Color::Rgb(42,42,42));
    set_px(buf, px - pot_r - 2, ty, Color::Rgb(42,42,42));
    for &cpx in &[cx + tw/4 - 2, cx + tw/4 + 3] {
        for dy in 0..2isize { for dx in 0..3isize {
            set_px(buf, cpx + dx, ty - dy, Color::Rgb(30,30,30));
        }}
    }
}

fn draw_shoji_frame(buf: &mut PixBuf, pw: usize, ph: usize) {
    let panel_w  = pw * 10 / 100;
    let paper    = (238u8, 232u8, 210u8);
    let paper_dk = Color::Rgb(210, 205, 185);
    let beam     = Color::Rgb(38, 24, 10);
    let beam_lt  = Color::Rgb(62, 40, 18);
    let grid_w   = (panel_w / 4).max(2);
    let grid_h   = (ph / 6).max(2);

    for side in 0..2usize {
        let x0 = if side == 0 { 0usize } else { pw.saturating_sub(panel_w) };
        let x1 = (x0 + panel_w).min(pw);
        for py in 0..ph {
            for ppx in x0..x1 {
                let gx = (ppx - x0) % grid_w == 0;
                let gy = py % grid_h == 0;
                let v  = hash(ppx as u64 * 3 + py as u64 * 7 + side as u64 * 999);
                let n  = (v % 8) as u8;
                buf[py][ppx] = Some(if gx || gy {
                    paper_dk
                } else {
                    Color::Rgb(paper.0.saturating_add(n/2), paper.1.saturating_add(n/3), paper.2)
                });
            }
        }
        let pillar_x = if side == 0 { x1.saturating_sub(3) } else { x0 };
        for py in 0..ph {
            for dx in 0..3usize {
                let ppx = pillar_x + dx;
                if ppx < pw { buf[py][ppx] = Some(if dx == 1 { beam_lt } else { beam }); }
            }
        }
    }
    let lintel_h = (ph * 4 / 100).max(2);
    for py in 0..lintel_h {
        for ppx in 0..pw {
            buf[py][ppx] = Some(if py == lintel_h - 1 { Color::Rgb(22,14,6) } else { beam });
        }
    }
}

pub(super) fn fill_blossom(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let horizon_y = ph * 63 / 100;

    // Sky: pale blue at top → warm peachy pink at horizon
    for py in 0..horizon_y {
        let f = py as f64 / horizon_y as f64;
        for ppx in 0..pw {
            buf[py][ppx] = Some(Color::Rgb(
                (152.0 + f * 78.0) as u8,
                (188.0 + f * 38.0) as u8,
                (225.0 - f * 45.0) as u8,
            ));
        }
    }

    // Tree geometry defined early so mountain peak can sit 10px above tree line
    let tree_base = ph * 76 / 100 + 2;
    let trees: &[(f64, f64)] = &[
        (0.08, 0.68), (0.20, 0.82), (0.35, 0.93), (0.51, 0.88),
        (0.66, 0.84), (0.79, 0.76), (0.93, 0.64),
    ];
    let max_sf   = trees.iter().map(|&(_, sf)| sf).fold(0.0f64, f64::max);
    let th_max   = (max_sf * ph as f64 * 0.115).max(5.0) as isize;
    let bl_max   = (max_sf * ph as f64 * 0.09).max(4.0) as isize;
    let tree_line = tree_base as isize - th_max - bl_max;

    // Mount Fuji — stone body first, snow overlay second
    let fcx   = pw as isize * 38 / 100;
    let ftop  = (tree_line - 10).max(0) as usize;
    let fbase = horizon_y;
    let fh    = fbase.saturating_sub(ftop);

    // t=0 at peak, t=1 at base. t^0.55: grows fast near 0 → blunt peak, not needle.
    // Left base reaches x=0, right keeps same asymmetry ratio (0.28/0.19 ≈ 1.47).
    let profile = |t: f64, spread: f64| -> isize {
        (t.powf(0.55) * spread * pw as f64) as isize
    };
    // Smooth edge: only commit jitter when both neighbours agree → ±1 max
    let edge_n = |seed: u64| -> isize {
        let a = (hash(seed)     % 3) as isize - 1;
        let b = (hash(seed + 1) % 3) as isize - 1;
        if a == b { a } else { 0 }
    };

    // Pass 1 — full stone body
    for py in ftop..fbase {
        let t        = (py - ftop) as f64 / fh.max(1) as f64;
        let left_hw  = profile(t, 0.28) + edge_n(py as u64 * 13 + 91);
        let right_hw = profile(t, 0.19) + edge_n(py as u64 * 17 + 43);
        for dx in -left_hw..=right_hw {
            let ppxi = fcx + dx;
            if ppxi < 0 || ppxi as usize >= pw { continue; }
            let ppx = ppxi as usize;
            let v   = hash(ppx as u64 * 3 + py as u64 * 7 + 222);
            let n   = (v % 8) as f64;
            let is_shadow = dx > right_hw / 2 && t > 0.15;
            let sf  = if is_shadow { 0.74 } else { 1.0 };
            buf[py][ppx] = Some(Color::Rgb(
                ((58.0 + t * 20.0 + n) * sf).min(255.0) as u8,
                ((68.0 + t * 22.0 + n) * sf).min(255.0) as u8,
                ((92.0 + t * 28.0 + n) * sf).min(255.0) as u8,
            ));
        }
    }

    // Pass 2 — snow: probabilistic over the stone pixel set, row-first.
    // prob=1 near peak, quadratic falloff below threshold.
    // ~1-in-7 columns are gullies: deeper threshold + wider fade → visible streaks.
    for py in ftop..fbase {
        let t        = (py - ftop) as f64 / fh.max(1) as f64;
        if t > 0.56 { break; }
        let left_hw  = profile(t, 0.28) + edge_n(py as u64 * 13 + 91);
        let right_hw = profile(t, 0.19) + edge_n(py as u64 * 17 + 43);

        for dx in -left_hw..=right_hw {
            let ppxi = fcx + dx;
            if ppxi < 0 || ppxi as usize >= pw { continue; }
            let ppx = ppxi as usize;

            let gully     = hash(ppx as u64 * 31 + 7777) % 7 == 0;
            let threshold = if gully { 0.30 } else { 0.22 };
            let fade_w    = if gully { 0.22 } else { 0.14 };

            let prob = if t <= threshold {
                1.0
            } else if t <= threshold + fade_w {
                let f = (t - threshold) / fade_w;
                (1.0 - f).powi(2)
            } else {
                0.0
            };
            if prob <= 0.0 { continue; }

            let v = hash(ppx as u64 * 17 + py as u64 * 23 + 555);
            if prob < 1.0 && (v % 1000) as f64 >= prob * 1000.0 { continue; }

            let n  = (hash(ppx as u64 * 5 + py as u64 * 11 + 333) % 6) as f64;
            let is_shadow = dx > right_hw / 2;
            let s  = if is_shadow { 14.0 } else { 0.0 };
            buf[py][ppx] = Some(Color::Rgb(
                ((237.0 + n * 0.5 - s).max(0.0).min(255.0)) as u8,
                ((242.0 + n * 0.4 - s).max(0.0).min(255.0)) as u8,
                ((252.0             - s).max(0.0).min(255.0)) as u8,
            ));
        }
    }

    // Ground
    for py in horizon_y..ph {
        let depth = (py - horizon_y) as f64 / (ph - horizon_y).max(1) as f64;
        for ppx in 0..pw {
            let v = hash(ppx as u64 * 5 + py as u64 * 11 + 777);
            let n = (v % 8) as f64;
            buf[py][ppx] = Some(Color::Rgb(
                (42.0 + depth * 20.0 + n) as u8,
                (62.0 + depth * 15.0 + n) as u8,
                (35.0 + depth * 10.0 + n) as u8,
            ));
        }
    }

    // Cherry blossom orchard — branched trees
    for (i, &(xf, sf)) in trees.iter().enumerate() {
        let tx = (xf * pw as f64) as isize;
        draw_cherry_tree_b(buf, ph, tx, tree_base, sf, (i as u64 + 1) * 10000);
    }

    // Balcony posts and rails
    let rail_y   = ph * 87 / 100;
    let post_top = rail_y.saturating_sub(ph * 12 / 100);
    let blt  = Color::Rgb(118, 72, 34);
    let bdk  = Color::Rgb(78, 46, 18);
    let flt  = (128u8, 80u8, 36u8);
    let fdk  = (92u8, 56u8, 22u8);
    let phw  = (pw / 90).max(1) as isize;
    for k in 0..=5usize {
        let pxc = (pw * k / 5) as isize;
        for py in post_top..rail_y {
            for dx in -phw..=phw {
                set_px(buf, pxc + dx, py as isize, if dx.abs() == phw { bdk } else { blt });
            }
        }
    }
    let mid_rail = post_top + (rail_y - post_top) / 2;
    for &ry in &[post_top, mid_rail, rail_y] {
        for h in 0..3usize {
            let py = ry + h;
            if py >= ph { continue; }
            for ppx in 0..pw { buf[py][ppx] = Some(if h == 0 { blt } else { bdk }); }
        }
    }
    let pw_plank = (pw / 9).max(3);
    for py in (rail_y + 3)..ph {
        let d = (py - rail_y) as f64 / (ph - rail_y).max(1) as f64;
        for ppx in 0..pw {
            let v = hash((ppx / pw_plank) as u64 * 13 + py as u64 * 7 + 5555);
            let n = (v % 12) as u8;
            buf[py][ppx] = Some(if ppx % pw_plank == 0 { bdk } else if d < 0.5 {
                Color::Rgb(flt.0.saturating_add(n / 3), flt.1.saturating_add(n / 4), flt.2.saturating_add(n / 5))
            } else {
                Color::Rgb(fdk.0.saturating_add(n / 3), fdk.1.saturating_add(n / 4), fdk.2.saturating_add(n / 5))
            });
        }
    }

    draw_tea_setup(buf, pw, ph);

    // Falling petals
    let pl  = Color::Rgb(255, 215, 228);
    let pm  = Color::Rgb(245, 178, 202);
    let pd  = Color::Rgb(222, 142, 168);
    let n_petals = (pw * ph / 55).max(18).min(90);
    let pcols = [pl, pm, pd, Color::Rgb(255, 235, 242)];
    for i in 0..n_petals {
        let h1  = hash(i as u64 + 2001);
        let h2  = hash(i as u64 + 3001);
        let xb  = (h1 % pw as u64) as usize;
        let ys  = (h2 % ph as u64) as usize;
        let spd = 1 + (h1 >> 20) % 2;
        let sa  = 2.0 + (h2 >> 10 & 0xf) as f64 * 0.35;
        let sf  = 0.028 + (h1 >> 15 & 0x7) as f64 * 0.005;
        let phs = (h2 >> 8 & 0x3f) as f64;
        let x   = (xb as f64 + (tick as f64 * sf + phs).sin() * sa) as isize;
        let y   = ((ys + tick as usize * spd as usize) % ph) as isize;
        if y >= (rail_y + 3) as isize { continue; }
        let col = pcols[(h1 as usize) % pcols.len()];
        set_px(buf, x, y, col);
        if (h1 >> 24) % 3 != 0 { set_px(buf, x + 1, y, col); }
    }

    // Shoji room frame — drawn last, over everything
    draw_shoji_frame(buf, pw, ph);
}

use super::*;

fn draw_fireplace(buf: &mut PixBuf, pw: usize, ph: usize) {
    let side_w   = (pw * 19 / 100).max(4);
    let mantel_h = (ph * 11 / 100).max(2);
    let hearth_y = ph.saturating_sub((ph * 9 / 100).max(2));
    let fire_x0  = side_w;
    let fire_x1  = pw.saturating_sub(side_w);

    let mortar   = Color::Rgb(52, 47, 42);

    let brick_h = (ph / 18).max(2);
    let brick_w = (side_w / 3).max(3);

    // Brick columns
    for side in 0..2usize {
        let xs = if side == 0 { 0 } else { pw.saturating_sub(side_w) };
        for py in mantel_h..hearth_y {
            for ppx in xs..(xs + side_w).min(pw) {
                let row  = (py - mantel_h) / brick_h;
                let off  = if row % 2 == 0 { 0 } else { brick_w / 2 };
                let col_b = (ppx - xs + off) % brick_w;
                let row_b = (py - mantel_h) % brick_h;
                let v = hash(((row * 13 + (ppx - xs + off) / brick_w) as u64).wrapping_add(77));
                let c = if row_b == 0 || col_b == 0 {
                    mortar
                } else if row_b == 1 {
                    // shadow just below mortar joint
                    Color::Rgb(
                        90u8.saturating_add((v % 15) as u8),
                        45u8.saturating_add(((v >> 4) % 10) as u8),
                        30u8.saturating_add(((v >> 8) % 8) as u8),
                    )
                } else {
                    Color::Rgb(
                        135u8.saturating_add((v % 25) as u8),
                        72u8.saturating_add(((v >> 4) % 18) as u8),
                        52u8.saturating_add(((v >> 8) % 14) as u8),
                    )
                };
                if py < buf.len() && ppx < buf[0].len() { buf[py][ppx] = Some(c); }
            }
        }
    }

    // Mantelpiece — warm oak wood grain
    for py in 0..mantel_h {
        for ppx in 0..pw {
            if py >= buf.len() || ppx >= buf[0].len() { continue; }
            let grain = hash((ppx as u64 / 2).wrapping_add((py as u64 * 3).wrapping_mul(17)));
            let grain_line = (py * pw + ppx / 3) % 7 == 0;
            let (r, g, b) = if py == mantel_h - 1 {
                (52u8, 31u8, 12u8)  // dark underside shadow
            } else if grain_line {
                (72u8.saturating_add((grain % 10) as u8), 43u8.saturating_add((grain % 8) as u8), 16u8.saturating_add((grain % 5) as u8))
            } else {
                (88u8.saturating_add((grain % 18) as u8), 54u8.saturating_add((grain % 13) as u8), 22u8.saturating_add((grain % 9) as u8))
            };
            buf[py][ppx] = Some(Color::Rgb(r, g, b));
        }
    }

    // Hearth floor — stone flags outside, carpet in centre
    let carpet_x0 = fire_x0.saturating_sub(fire_x0 / 3);
    let carpet_x1 = (fire_x1 + (pw - fire_x1) / 3).min(pw);
    for py in hearth_y..ph {
        for ppx in 0..pw {
            if py >= buf.len() || ppx >= buf[0].len() { continue; }
            let on_carpet = ppx >= carpet_x0 && ppx < carpet_x1;
            if on_carpet {
                let cx = ppx - carpet_x0;
                let cy = py - hearth_y;
                let cw = carpet_x1 - carpet_x0;
                let ch = ph - hearth_y;
                let border = cx == 0 || cx == cw.saturating_sub(1) || cy == 0 || cy == ch.saturating_sub(1);
                let inner_border = cx == 2 || cx == cw.saturating_sub(3) || cy == 2 || cy == ch.saturating_sub(3);
                let v = hash(ppx as u64 * 7 + py as u64 * 13 + 8888);
                let noise = (v % 12) as u8;
                buf[py][ppx] = Some(if border || inner_border {
                    Color::Rgb(130u8.saturating_add(noise / 2), 40u8.saturating_add(noise / 4), 18u8.saturating_add(noise / 4))
                } else {
                    Color::Rgb(105u8.saturating_add(noise), 28u8.saturating_add(noise / 3), 18u8.saturating_add(noise / 3))
                });
            } else {
                let v = hash(((ppx / 5) as u64).wrapping_add((py * 53) as u64));
                buf[py][ppx] = Some(Color::Rgb(
                    78u8.saturating_add((v % 18) as u8),
                    76u8.saturating_add(((v >> 4) % 14) as u8),
                    72u8.saturating_add(((v >> 8) % 11) as u8),
                ));
            }
        }
    }

    // Logs — two overlapping logs sitting at the base of the fire
    let log_h  = (ph * 5 / 100).max(2);
    let log_y0 = hearth_y.saturating_sub(log_h + 1);
    let fire_w = fire_x1 - fire_x0;
    let log_hw = fire_w / 2;
    let logs: &[(isize, u8, u8, u8)] = &[
        (fire_x0 as isize + fire_w as isize / 3,     58, 32, 12),
        (fire_x0 as isize + fire_w as isize * 2 / 3, 48, 26, 10),
    ];
    for &(cx, lr, lg, lb) in logs {
        for dy in 0..log_h {
            let py = log_y0 + dy;
            if py >= ph { continue; }
            let ry = dy as f64 / log_h as f64;
            let half_w = (log_hw as f64 * (1.0 - (ry * 2.0 - 1.0).powi(2)).sqrt()) as isize;
            for dx in -half_w..=half_w {
                let px = cx + dx;
                if px < fire_x0 as isize || px as usize >= fire_x1 { continue; }
                let v = hash(px as u64 * 5 + py as u64 * 11 + 3030);
                let bark = (v % 3 == 0) as u8 * 12;
                let ember = if dy == 0 { 30u8 } else { 0u8 };
                buf[py][px as usize] = Some(Color::Rgb(
                    lr.saturating_add(bark).saturating_add(ember),
                    lg.saturating_add(bark / 2),
                    lb.saturating_add(bark / 3),
                ));
            }
        }
    }

    // Iron grate — vertical bars across lower portion of fire opening
    let grate_h  = (ph * 10 / 100).max(3);
    let grate_y0 = hearth_y.saturating_sub(grate_h);
    let bar_w    = 1usize;
    let gap_w    = (fire_x1 - fire_x0) / 10;
    let gap_w    = gap_w.max(2);
    let iron_hi  = Color::Rgb(48, 45, 42);
    let iron_dk  = Color::Rgb(22, 20, 18);
    // horizontal rail at top and bottom of grate
    for ppx in fire_x0..fire_x1 {
        if grate_y0 < buf.len() { buf[grate_y0][ppx] = Some(iron_hi); }
        if hearth_y < buf.len() && hearth_y < ph { buf[hearth_y.min(ph-1)][ppx] = Some(iron_dk); }
    }
    // vertical bars
    let mut bx = fire_x0 + gap_w / 2;
    while bx + bar_w <= fire_x1 {
        for py in grate_y0..hearth_y {
            if py >= buf.len() { continue; }
            for dx in 0..bar_w {
                let px = bx + dx;
                if px < buf[0].len() {
                    buf[py][px] = Some(if dx == 0 { iron_hi } else { iron_dk });
                }
            }
        }
        bx += gap_w;
    }
}

pub(super) fn fill_fire(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let side_w   = (pw * 19 / 100).max(4);
    let mantel_h = (ph * 11 / 100).max(2);
    let hearth_y = ph.saturating_sub((ph * 9 / 100).max(2));
    let fire_x0  = side_w;
    let fire_x1  = pw.saturating_sub(side_w);
    let fire_pw  = fire_x1.saturating_sub(fire_x0);
    let fire_ph  = hearth_y.saturating_sub(mantel_h);

    // Dark room background
    for py in 0..ph { for ppx in 0..pw { buf[py][ppx] = Some(Color::Rgb(5, 4, 3)); } }

    // Back wall of fireplace — soot-stained, darker than room to push it back
    for py in mantel_h..hearth_y {
        for ppx in fire_x0..fire_x1 {
            let v = hash(ppx as u64 * 3 + py as u64 * 7 + 999);
            buf[py][ppx] = Some(Color::Rgb(
                11u8.saturating_add((v % 4) as u8),
                9u8.saturating_add(((v >> 4) % 3) as u8),
                8u8.saturating_add(((v >> 8) % 3) as u8),
            ));
        }
    }

    // Fire — hash-based crackle: each column flickers independently, no sine sweep
    let crackle_t = tick / 4;
    let blend_f   = (tick % 4) as f64 / 4.0;
    for ppx in fire_x0..fire_x1 {
        let local_x = ppx - fire_x0;
        let cx      = (local_x as f64 - fire_pw as f64 / 2.0).abs() / (fire_pw as f64 / 2.0);
        let arch    = (1.0 - cx * 0.65).max(0.0);

        // Current and next crackle state blended — independent per column
        let ha = hash(ppx as u64 * 11 + crackle_t * 37 + 101);
        let hb = hash(ppx as u64 * 11 + (crackle_t + 1) * 37 + 101);
        let ca = (ha % 100) as f64 / 100.0 * 0.30 - 0.09;
        let cb = (hb % 100) as f64 / 100.0 * 0.30 - 0.09;
        // Slight neighbour influence so adjacent columns aren't totally unrelated
        let hn = hash((ppx as u64 + 1) * 11 + crackle_t * 37 + 101);
        let cn = (hn % 100) as f64 / 100.0 * 0.30 - 0.09;
        let crackle = (ca * (1.0 - blend_f) + cb * blend_f) * 0.75 + cn * 0.25;

        let height = ((arch + crackle).clamp(0.1, 1.0) * 0.58 * fire_ph as f64) as usize;
        let top    = hearth_y.saturating_sub(height);

        for py in top..hearth_y {
            let f = (py - top) as f64 / height.max(1) as f64;
            buf[py][ppx] = Some(
                if f < 0.20 { let v = f/0.20; Color::Rgb(255, (210.0*v+45.0) as u8, (90.0*(1.0-v)) as u8) }
                else if f < 0.60 { let v = (f-0.20)/0.40; Color::Rgb(255, (45.0*(1.0-v)) as u8, 0) }
                else { Color::Rgb(((1.0-(f-0.60)/0.40)*240.0).max(0.0) as u8, 0, 0) }
            );
        }
    }

    // Sparks — brief bright pixels near flame tips
    let n_sparks = (fire_pw / 2).max(3);
    for i in 0..n_sparks {
        let hs = hash(i as u64 * 17 + tick / 5 * 31 + 555);
        if hs % 4 != 0 { continue; }
        let sx = fire_x0 + (hs >> 10) as usize % fire_pw;
        let sy = mantel_h + (hs >> 20) as usize % (fire_ph * 2 / 5);
        set_px(buf, sx as isize, sy as isize, match (hs >> 5) % 3 {
            0 => Color::Rgb(255, 210, 60),
            1 => Color::Rgb(255, 150, 25),
            _ => Color::Rgb(255, 255, 190),
        });
    }

    // Depth — shadow gradient at inner edges of opening, simulating column thickness
    let shadow_w = (fire_pw / 6).max(3);
    for py in mantel_h..hearth_y {
        for dx in 0..shadow_w {
            let alpha = (1.0 - dx as f64 / shadow_w as f64) * 0.78;
            for &px in &[fire_x0 + dx, fire_x1.saturating_sub(1 + dx)] {
                if px < buf[0].len() {
                    if let Some(Color::Rgb(r, g, b)) = buf[py][px] {
                        buf[py][px] = Some(Color::Rgb(
                            (r as f64 * (1.0 - alpha)) as u8,
                            (g as f64 * (1.0 - alpha)) as u8,
                            (b as f64 * (1.0 - alpha)) as u8,
                        ));
                    }
                }
            }
        }
    }

    // Fireplace structure drawn on top
    draw_fireplace(buf, pw, ph);
}

use super::*;

fn draw_mountains_and_buildings(buf: &mut PixBuf, pw: usize, ph: usize) {
    let ground_y = ph * 88 / 100;
    let sky_col = |py: usize| {
        let frac = py as f64 / ground_y as f64;
        Color::Rgb((8.0 + frac * 6.0) as u8, (12.0 + frac * 8.0) as u8, (22.0 + frac * 12.0) as u8)
    };

    // Mountains
    let m_base = ph * 72 / 100;
    for ppx in 0..pw {
        let xf = ppx as f64 / pw as f64;
        let r1 = ((xf * std::f64::consts::PI * 2.8).sin() * 0.5 + 0.5) * ph as f64 * 0.28;
        let r2 = ((xf * std::f64::consts::PI * 1.5 + 1.0).sin() * 0.5 + 0.5) * ph as f64 * 0.14;
        let ridge = (r1 + r2) as usize;
        let top = m_base.saturating_sub(ridge);
        for py in top..m_base.min(ground_y) {
            let d = (py - top) as f64 / ridge.max(1) as f64;
            buf[py][ppx] = Some(Color::Rgb(
                (38.0 + d * 12.0) as u8,
                (45.0 + d * 14.0) as u8,
                (72.0 + d * 20.0) as u8,
            ));
        }
    }

    let win     = Color::Rgb(210, 175, 85);
    let win_dim = Color::Rgb(125, 103, 42);
    let beam    = Color::Rgb(48,  32,  16);   // dark oak
    let mortar  = Color::Rgb(88,  82,  74);   // stone mortar
    let spire_c = (82u8, 77u8, 70u8);   // dark old stone
    let plant   = Color::Rgb(42,  95,  35);
    let pot     = Color::Rgb(140, 72,  32);

    // (x_frac, w_frac, h_frac, style)  0=tower, 1=hall, 2=spire
    let bldgs: &[(f64, f64, f64, u8)] = &[
        (0.00, 0.07, 0.40, 0),
        (0.06, 0.13, 0.26, 1),
        (0.17, 0.06, 0.55, 2),
        (0.22, 0.14, 0.30, 1),
        (0.35, 0.07, 0.46, 0),
        (0.41, 0.16, 0.23, 1),
        (0.57, 0.06, 0.44, 2),
        (0.62, 0.11, 0.32, 1),
        (0.72, 0.07, 0.48, 0),
        (0.78, 0.13, 0.27, 1),
        (0.90, 0.10, 0.42, 0),
    ];

    let b_base = ph * 78 / 100;
    for &(xf, wf, hf, style) in bldgs {
        let bx = (xf * pw as f64) as isize;
        let bw = ((wf * pw as f64) as usize).max(3);
        let bh = ((hf * ph as f64) as usize).max(4);
        let by = b_base.saturating_sub(bh) as isize;

        // Draw walls from building top all the way to ground
        let wall_bottom = ground_y.min(ph);
        match style {
            0 => {
                // Tower — light stone with coursed masonry
                for py in (by as usize)..wall_bottom {
                    for ppx in (bx as usize)..(bx as usize + bw).min(pw) {
                        let v   = hash(ppx as u64 * 3 + py as u64 * 7 + 101);
                        let row = py.wrapping_sub(by as usize);
                        let is_mortar = row % 3 == 0 || ppx % 4 == (if (row / 3) % 2 == 0 { 0 } else { 2 });
                        buf[py][ppx] = Some(if is_mortar {
                            mortar
                        } else {
                            Color::Rgb(
                                158u8.saturating_add((v % 18) as u8),
                                150u8.saturating_add(((v >> 4) % 14) as u8),
                                136u8.saturating_add(((v >> 8) % 11) as u8),
                            )
                        });
                    }
                }
                // Battlements
                let merlon_w = (bw / 4).max(1);
                let crenel_h = (bh / 5).max(1);
                let mut toggle = true;
                let mut cx = bx;
                while cx < bx + bw as isize {
                    let seg_end = (cx + merlon_w as isize).min(bx + bw as isize);
                    if !toggle {
                        for py in (by - crenel_h as isize)..by {
                            for ppx in cx..seg_end {
                                if py >= 0 && ppx >= 0 && (ppx as usize) < pw && (py as usize) < ph {
                                    buf[py as usize][ppx as usize] = Some(sky_col(py as usize));
                                }
                            }
                        }
                    }
                    toggle = !toggle;
                    cx = seg_end;
                }
                if bw >= 4 && bh >= 6 {
                    let wx = bx + bw as isize / 2;
                    let wy = by + bh as isize / 3;
                    set_px(buf, wx, wy,     win);
                    set_px(buf, wx, wy + 1, win_dim);
                }
            }
            2 => {
                // Spire — dark old stone tower with needle
                for py in (by as usize)..wall_bottom {
                    for ppx in (bx as usize)..(bx as usize + bw).min(pw) {
                        let v = hash(ppx as u64 * 5 + py as u64 * 11 + 303);
                        buf[py][ppx] = Some(Color::Rgb(
                            spire_c.0.saturating_add((v % 10) as u8),
                            spire_c.1.saturating_add(((v >> 4) % 8) as u8),
                            spire_c.2.saturating_add(((v >> 8) % 7) as u8),
                        ));
                    }
                }
                let spire_h = (bh * 3 / 4) as isize;
                let half_bw = (bw as isize / 2).max(1);
                let cx = bx + bw as isize / 2;
                for dy in 0..spire_h {
                    let hw = (half_bw as f64 * (spire_h - dy) as f64 / spire_h as f64 * 0.6) as isize;
                    for dx in -hw..=hw {
                        let v = hash((cx + dx).unsigned_abs() as u64 * 7 + (by - spire_h + dy).unsigned_abs() as u64 * 13 + 404);
                        set_px(buf, cx + dx, by - spire_h + dy, Color::Rgb(
                            spire_c.0.saturating_add((v % 8) as u8),
                            spire_c.1.saturating_add(((v >> 4) % 6) as u8),
                            spire_c.2.saturating_add(((v >> 8) % 5) as u8),
                        ));
                    }
                }
                if bw >= 3 && bh >= 8 {
                    set_px(buf, bx + bw as isize / 2, by + bh as isize * 2 / 5, win);
                }
            }
            _ => {
                // Hall — half-timbered: cream plaster + dark oak beams
                // Per-building colour variation seeded from position
                let bseed = hash(bx.unsigned_abs() as u64 * 31 + 999);
                let br_off = (bseed % 30) as u8;
                let bg_off = ((bseed >> 4) % 22) as u8;
                let bb_off = ((bseed >> 8) % 18) as u8;
                for py in (by as usize)..wall_bottom {
                    for ppx in (bx as usize)..(bx as usize + bw).min(pw) {
                        let v = hash(ppx as u64 * 5 + py as u64 * 9 + 202);
                        buf[py][ppx] = Some(Color::Rgb(
                            (175u8.saturating_add(br_off)).saturating_add((v % 12) as u8),
                            (165u8.saturating_add(bg_off)).saturating_add(((v >> 4) % 10) as u8),
                            (148u8.saturating_add(bb_off)).saturating_add(((v >> 8) % 8) as u8),
                        ));
                    }
                }
                // Vertical studs: corners + thirds
                for &dx in &[0isize, bw as isize / 3, bw as isize * 2 / 3, bw as isize - 1] {
                    for py in (by as usize)..wall_bottom {
                        set_px(buf, bx + dx, py as isize, beam);
                    }
                }
                // Horizontal rails: top, thirds
                for &dy in &[0isize, bh as isize / 3, bh as isize * 2 / 3] {
                    for ppx in (bx as usize)..(bx as usize + bw).min(pw) {
                        set_px(buf, ppx as isize, by + dy, beam);
                    }
                }
                // Pitched roof — wide at building top, peak at top
                let roof_h = (bh / 3).max(2) as isize;
                let cx_r = bx + bw as isize / 2;
                for dy in 0..roof_h {
                    let hw = (bw as f64 * 0.5 * (roof_h - dy) as f64 / roof_h as f64) as isize;
                    for dx in -hw..=hw {
                        let v = hash((cx_r + dx).unsigned_abs() as u64 * 3 + (by - dy).unsigned_abs() as u64 * 7 + 505);
                        let edge = dx.abs() == hw;
                        set_px(buf, cx_r + dx, by - dy, if edge {
                            beam
                        } else {
                            Color::Rgb(
                                62u8.saturating_add((v % 12) as u8),
                                45u8.saturating_add(((v >> 4) % 9) as u8),
                                30u8.saturating_add(((v >> 8) % 7) as u8),
                            )
                        });
                    }
                }
                // Ridge beam at peak
                set_px(buf, cx_r, by - roof_h, beam);
                // Windows + plant pots
                if bw >= 5 && bh >= 5 {
                    let wy   = by + bh as isize / 3 + 1;
                    let step = (bw / 3).max(2) as isize;
                    let mut wx = bx + step / 2;
                    let mut pot_toggle = false;
                    while wx < bx + bw as isize - 1 {
                        set_px(buf, wx, wy,     win);
                        set_px(buf, wx, wy + 1, win_dim);
                        if pot_toggle {
                            set_px(buf, wx, wy - 1, plant);
                            set_px(buf, wx, wy + 2, pot);
                        }
                        pot_toggle = !pot_toggle;
                        wx += step;
                    }
                }
            }
        }

        // Door — all building styles, centered at base
        let door_w = (bw / 4).max(2);
        let door_h = (bh / 4).max(3);
        let door_x = bx + bw as isize / 2 - door_w as isize / 2;
        let door_top = ground_y as isize - door_h as isize;
        let door_col = Color::Rgb(14, 10, 6);
        for py in door_top..ground_y as isize {
            let row_from_top = (py - door_top) as usize;
            for dx in 0..door_w as isize {
                // arch: round top corners on first row
                let is_corner = row_from_top == 0 && (dx == 0 || dx == door_w as isize - 1);
                if !is_corner {
                    set_px(buf, door_x + dx, py, door_col);
                }
            }
        }
    }
}

pub(super) fn fill_rain(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let ground_y = ph * 88 / 100;

    // Sky
    for py in 0..ground_y {
        let frac = py as f64 / ground_y as f64;
        for ppx in 0..pw {
            buf[py][ppx] = Some(Color::Rgb((8.0 + frac*6.0) as u8, (12.0 + frac*8.0) as u8, (22.0 + frac*12.0) as u8));
        }
    }

    // Cobblestone ground
    let cobble_h = (ph / 22).max(2);
    let cobble_w = (pw / 14).max(3);
    for py in ground_y..ph {
        for ppx in 0..pw {
            let row     = (py - ground_y) / cobble_h;
            let offset  = if row % 2 == 0 { 0 } else { cobble_w / 2 };
            let col_b   = (ppx + offset) % cobble_w;
            let row_b   = (py - ground_y) % cobble_h;
            buf[py][ppx] = Some(if row_b == 0 || col_b == 0 {
                Color::Rgb(14, 16, 14) // mortar
            } else {
                let v = hash(((row * 89 + (ppx + offset) / cobble_w) as u64).wrapping_add(301));
                Color::Rgb(
                    42u8.saturating_add((v % 22) as u8),
                    46u8.saturating_add(((v >> 4) % 20) as u8),
                    40u8.saturating_add(((v >> 8) % 16) as u8),
                )
            });
        }
    }

    draw_mountains_and_buildings(buf, pw, ph);

    let speed = 3usize;
    let spacing = 12usize;
    for ppx in 0..pw {
        let col_offset = (hash(ppx as u64 + 7) % spacing as u64) as usize;
        let streak = 3 + (hash(ppx as u64 + 99) % 4) as usize;
        let base = (tick as usize * speed + col_offset * ph / spacing) % ph;
        for s in 0..streak {
            let py = (base + s) % ph;
            if py < ground_y {
                let alpha = 1.0 - s as f64 / streak as f64;
                buf[py][ppx] = Some(Color::Rgb((80.0*alpha) as u8, (130.0*alpha) as u8, (255.0*alpha) as u8));
            }
        }
    }

    // Puddle ripples on cobblestones
    for py in ground_y..ph {
        for ppx in 0..pw {
            let ripple = (ppx as f64 * 0.4 + tick as f64 * 0.15).sin();
            if ripple > 0.7 { buf[py][ppx] = Some(Color::Rgb(28, 52, 75)); }
        }
    }
}

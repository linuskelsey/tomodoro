use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

// ── Pixel buffer ──────────────────────────────────────────────────────────────

type PixBuf = Vec<Vec<Option<Color>>>;

fn new_buf(pw: usize, ph: usize) -> PixBuf {
    vec![vec![None; pw]; ph]
}

fn px(buf: &PixBuf, x: usize, y: usize) -> Option<Color> {
    buf.get(y).and_then(|r| r.get(x)).copied().flatten()
}

fn lum(c: Option<Color>) -> f32 {
    match c {
        Some(Color::Rgb(r, g, b)) => 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32,
        _ => 0.0,
    }
}

// ── Render mode ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode { Half, Quarter, Braille }

impl RenderMode {
    fn next(self) -> Self {
        match self { Self::Half => Self::Quarter, Self::Quarter => Self::Braille, Self::Braille => Self::Half }
    }
    fn prev(self) -> Self {
        match self { Self::Half => Self::Braille, Self::Quarter => Self::Half, Self::Braille => Self::Quarter }
    }
    // Pixel dimensions for a char_w × char_h terminal area
    fn pw(self, char_w: usize) -> usize {
        match self { Self::Half => char_w, Self::Quarter | Self::Braille => char_w * 2 }
    }
    fn ph(self, char_h: usize) -> usize {
        match self { Self::Half | Self::Quarter => char_h * 2, Self::Braille => char_h * 4 }
    }
}

// ── Half-block renderer (1×2 pixels per char) ─────────────────────────────────

fn render_half(buf: &PixBuf, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
    to_lines(char_w, char_h, |col, row| {
        let t = px(buf, col, row * 2);
        let b = px(buf, col, row * 2 + 1);
        match (t, b) {
            (None,    None   ) => (' ', Style::new()),
            (Some(c), None   ) => ('▀', Style::new().fg(c)),
            (None,    Some(c)) => ('▄', Style::new().fg(c)),
            (Some(a), Some(b)) if a == b => ('█', Style::new().fg(a)),
            (Some(a), Some(b)) => ('▀', Style::new().fg(a).bg(b)),
        }
    })
}

// ── Quarter-block renderer (2×2 pixels per char) ──────────────────────────────

// Bit order: [3]=TL [2]=TR [1]=BL [0]=BR
const Q: [char; 16] = [
    ' ', '▗', '▖', '▄', '▝', '▐', '▞', '▟',
    '▘', '▚', '▌', '▙', '▀', '▜', '▛', '█',
];

fn render_quarter(buf: &PixBuf, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
    to_lines(char_w, char_h, |col, row| {
        let tl = px(buf, col*2,   row*2);
        let tr = px(buf, col*2+1, row*2);
        let bl = px(buf, col*2,   row*2+1);
        let br = px(buf, col*2+1, row*2+1);
        let pixels = [tl, tr, bl, br];
        let ls = pixels.map(lum);

        // Median split: 2 brightest → fg, 2 darkest → bg
        let mut s = ls;
        s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let thresh = (s[1] + s[2]) / 2.0;
        let on = ls.map(|l| l > thresh || (l == thresh && l > 0.0));

        let idx = (on[0] as usize)<<3 | (on[1] as usize)<<2 | (on[2] as usize)<<1 | on[3] as usize;
        let ch = Q[idx];

        let (mut fr, mut fg, mut fb, mut fn_) = (0u32, 0, 0, 0);
        let (mut br_, mut bg, mut bb, mut bn) = (0u32, 0, 0, 0);
        for (i, c) in pixels.iter().enumerate() {
            if let Some(Color::Rgb(r, g, b)) = c {
                if on[i] { fr += *r as u32; fg += *g as u32; fb += *b as u32; fn_ += 1; }
                else      { br_ += *r as u32; bg += *g as u32; bb += *b as u32; bn += 1; }
            }
        }
        let style = match (fn_ > 0, bn > 0) {
            (true,  true ) => Style::new()
                .fg(Color::Rgb((fr/fn_) as u8, (fg/fn_) as u8, (fb/fn_) as u8))
                .bg(Color::Rgb((br_/bn) as u8, (bg/bn)  as u8, (bb/bn)  as u8)),
            (true,  false) => Style::new()
                .fg(Color::Rgb((fr/fn_) as u8, (fg/fn_) as u8, (fb/fn_) as u8)),
            (false, true ) => Style::new()
                .bg(Color::Rgb((br_/bn) as u8, (bg/bn)  as u8, (bb/bn)  as u8)),
            (false, false) => Style::new(),
        };
        (ch, style)
    })
}

// ── Braille renderer (2×4 pixels per char) ────────────────────────────────────
//
// Braille dot layout (Unicode graphical ordering):
//   col0 col1
//    1    4     row 0
//    2    5     row 1
//    3    6     row 2
//    7    8     row 3
// Codepoint = U+2800 + bitmask

fn render_braille(buf: &PixBuf, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
    // bit positions for each of the 8 sub-pixels in a 2×4 block
    const BIT: [[u8; 2]; 4] = [
        [0x01, 0x08],  // row 0: dot1, dot4
        [0x02, 0x10],  // row 1: dot2, dot5
        [0x04, 0x20],  // row 2: dot3, dot6
        [0x40, 0x80],  // row 3: dot7, dot8
    ];

    to_lines(char_w, char_h, |col, row| {
        let mut mask: u8 = 0;
        let mut fr = 0u32; let mut fg = 0u32; let mut fb = 0u32; let mut fn_ = 0u32;
        let mut dr = 0u32; let mut dg = 0u32; let mut db = 0u32; // avg of all pixels for bg hint
        let mut dn = 0u32;

        let mut lums = [0f32; 8];
        let mut cols = [None::<Color>; 8];
        let mut k = 0;
        for dy in 0..4usize {
            for dx in 0..2usize {
                let c = px(buf, col*2+dx, row*4+dy);
                lums[k] = lum(c);
                cols[k] = c;
                k += 1;
            }
        }

        // Threshold = median of 8 luminances
        let mut s = lums;
        s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let thresh = (s[3] + s[4]) / 2.0;

        for dy in 0..4usize {
            for dx in 0..2usize {
                let i = dy * 2 + dx;
                let c = cols[i];
                if let Some(Color::Rgb(r, g, b)) = c {
                    dr += r as u32; dg += g as u32; db += b as u32; dn += 1;
                }
                if lums[i] > thresh || (lums[i] == thresh && lums[i] > 0.0) {
                    mask |= BIT[dy][dx];
                    if let Some(Color::Rgb(r, g, b)) = c {
                        fr += r as u32; fg += g as u32; fb += b as u32; fn_ += 1;
                    }
                }
            }
        }

        let ch = char::from_u32(0x2800 + mask as u32).unwrap_or(' ');

        // Fg = avg of lit pixels; bg = avg of all pixels (darker, fills gaps)
        let style = if fn_ > 0 && dn > 0 {
            let fg_c = Color::Rgb((fr/fn_) as u8, (fg/fn_) as u8, (fb/fn_) as u8);
            let bg_c = Color::Rgb(
                ((dr/dn) as u16).saturating_sub(30) as u8,
                ((dg/dn) as u16).saturating_sub(30) as u8,
                ((db/dn) as u16).saturating_sub(30) as u8,
            );
            Style::new().fg(fg_c).bg(bg_c)
        } else if fn_ > 0 {
            Style::new().fg(Color::Rgb((fr/fn_) as u8, (fg/fn_) as u8, (fb/fn_) as u8))
        } else {
            Style::new()
        };

        (ch, style)
    })
}

// ── Shared line builder ───────────────────────────────────────────────────────

fn to_lines(
    char_w: usize,
    char_h: usize,
    mut cell: impl FnMut(usize, usize) -> (char, Style),
) -> Vec<Line<'static>> {
    (0..char_h).map(|row| {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut run = String::new();
        let mut cur = Style::new();
        for col in 0..char_w {
            let (ch, sty) = cell(col, row);
            if sty == cur { run.push(ch); }
            else {
                if !run.is_empty() { spans.push(Span::styled(run.clone(), cur)); run.clear(); }
                cur = sty;
                run.push(ch);
            }
        }
        if !run.is_empty() { spans.push(Span::styled(run, cur)); }
        Line::from(spans)
    }).collect()
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn set_px(buf: &mut PixBuf, x: isize, y: isize, c: Color) {
    if y >= 0 && x >= 0 {
        if let Some(row) = buf.get_mut(y as usize) {
            if let Some(cell) = row.get_mut(x as usize) {
                *cell = Some(c);
            }
        }
    }
}

fn fill_r(buf: &mut PixBuf, x: isize, y: isize, w: usize, h: usize, c: Color) {
    for dy in 0..h as isize {
        for dx in 0..w as isize {
            set_px(buf, x + dx, y + dy, c);
        }
    }
}

// ── Hash helper ───────────────────────────────────────────────────────────────

fn hash(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn dispatch(mode: RenderMode, buf: &PixBuf, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
    match mode {
        RenderMode::Half    => render_half(buf, char_w, char_h),
        RenderMode::Quarter => render_quarter(buf, char_w, char_h),
        RenderMode::Braille => render_braille(buf, char_w, char_h),
    }
}

// ── Scene element drawing functions ──────────────────────────────────────────

fn draw_mountains_and_buildings(buf: &mut PixBuf, pw: usize, ph: usize) {
    let ground_y = ph * 88 / 100;
    let sky_col = |py: usize| {
        let frac = py as f64 / ground_y as f64;
        Color::Rgb((8.0 + frac * 6.0) as u8, (12.0 + frac * 8.0) as u8, (22.0 + frac * 12.0) as u8)
    };

    // Mountains — high contrast blue-grey against dark sky
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

    // Medieval buildings — visible warm-grey silhouettes with lit windows
    let sil = Color::Rgb(32, 34, 52);     // clearly lighter than sky
    let sil_dk = Color::Rgb(22, 24, 40);  // slightly darker variant for depth
    let win = Color::Rgb(200, 170, 80);   // warm bright window glow
    let win_dim = Color::Rgb(120, 100, 40);

    // (x_frac, w_frac, h_frac, style)  style: 0=tower, 1=hall, 2=spire
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

        fill_r(buf, bx, by, bw, b_base.saturating_sub(by as usize), sil);

        match style {
            0 => {
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
                    // Lit window
                    set_px(buf, bx + bw as isize / 2, by + bh as isize / 3, win);
                    set_px(buf, bx + bw as isize / 2, by + bh as isize / 3 + 1, win_dim);
                }
            }
            2 => {
                // Spire
                let spire_h = (bh * 3 / 4) as isize;
                let half_bw = (bw as isize / 2).max(1);
                let cx = bx + bw as isize / 2;
                for dy in 0..spire_h {
                    let hw = (half_bw as f64 * (spire_h - dy) as f64 / spire_h as f64 * 0.6) as isize;
                    for dx in -hw..=hw {
                        set_px(buf, cx + dx, by - spire_h + dy, sil_dk);
                    }
                }
                if bw >= 3 && bh >= 8 {
                    set_px(buf, bx + bw as isize / 2, by + bh as isize * 2 / 5, win);
                }
            }
            _ => {
                // Hall with multiple glowing windows
                if bw >= 5 && bh >= 5 {
                    let wy = by + bh as isize / 3;
                    let step = (bw / 3).max(2) as isize;
                    let mut wx = bx + step / 2;
                    while wx < bx + bw as isize - 1 {
                        set_px(buf, wx, wy, win);
                        set_px(buf, wx, wy + 1, win_dim);
                        wx += step;
                    }
                }
            }
        }
    }

    // Lamppost — foreground left of scene
    draw_lamppost(buf, pw, ph, b_base);
}

fn draw_lamppost(buf: &mut PixBuf, pw: usize, ph: usize, ground_y: usize) {
    let lx     = (pw as f64 * 0.22) as isize;
    let pole_h = (ph as f64 * 0.40) as usize;
    let pole_y = (ground_y.saturating_sub(pole_h)) as isize;
    let iron   = Color::Rgb(45, 42, 50);
    let glow   = Color::Rgb(240, 200, 100);
    let glow_d = Color::Rgb(160, 130, 55);
    let glow_f = Color::Rgb(90,  72,  28);

    // Pole
    for py in pole_y..ground_y as isize {
        set_px(buf, lx, py, iron);
    }
    // Curved arm at top
    set_px(buf, lx - 1, pole_y + 1, iron);
    set_px(buf, lx - 2, pole_y,     iron);
    set_px(buf, lx - 3, pole_y - 1, iron);

    // Lantern box
    let lant_x = lx - 4;
    let lant_y = pole_y - 3;
    fill_r(buf, lant_x - 1, lant_y, 3, 3, glow);

    // Glow halo radiating outward
    for dy in -4isize..=4 {
        for dx in -5isize..=5 {
            let dist = ((dx * dx + dy * dy * 2) as f64).sqrt();
            let cx = lant_x + dx;
            let cy = lant_y + 1 + dy;
            if dist > 1.5 && dist < 3.5 { set_px(buf, cx, cy, glow_d); }
            else if dist >= 3.5 && dist < 5.5 { set_px(buf, cx, cy, glow_f); }
        }
    }
    // Re-draw lantern on top of halo
    fill_r(buf, lant_x - 1, lant_y, 3, 3, glow);
}

fn trunk_half_w(t: f64, pw: usize) -> isize {
    // Uniform trunk — very slight taper, mostly cylindrical
    ((0.11 + t * 0.04) * pw as f64) as isize
}

fn draw_background_trees(buf: &mut PixBuf, pw: usize, ph: usize) {
    let ground_y = ph * 85 / 100;
    let trunk_col = Color::Rgb(32, 18, 7);

    // (x_frac, trunk_h_frac, canopy_r_frac, palette)
    let trees: &[(f64, f64, f64, usize)] = &[
        (0.07, 0.52, 0.11, 0),
        (0.17, 0.44, 0.09, 1),
        (0.27, 0.62, 0.13, 2),
        (0.73, 0.48, 0.10, 2),
        (0.83, 0.58, 0.12, 1),
        (0.93, 0.40, 0.08, 0),
    ];

    // Three autumn palettes: red, orange-gold, warm amber
    let palettes: &[[Color; 3]] = &[
        [Color::Rgb(180, 48, 8),  Color::Rgb(200, 70, 12),  Color::Rgb(155, 35, 5) ],
        [Color::Rgb(195, 120, 0), Color::Rgb(215, 148, 8),  Color::Rgb(170, 98,  0)],
        [Color::Rgb(162, 72, 15), Color::Rgb(185, 92,  20), Color::Rgb(140, 55, 10)],
    ];

    for &(xf, hf, rf, pal) in trees {
        let tx      = (xf * pw as f64) as isize;
        let trunk_h = (hf * ph as f64) as usize;
        let top_y   = ground_y.saturating_sub(trunk_h) as isize;
        let cr      = ((rf * pw as f64).max(3.0)) as isize;
        let cy      = top_y - cr / 2;
        let cols    = &palettes[pal % palettes.len()];

        // Thin trunk (1-2px)
        for py in top_y..ground_y as isize {
            set_px(buf, tx, py, trunk_col);
        }

        // Canopy — slightly elliptical blob with density variation
        for dy in -cr..=cr {
            for dx in -cr * 2..=cr * 2 {
                // Ellipse: squash x
                let ex = dx / 2;
                if ex * ex + dy * dy <= cr * cr {
                    let v = hash((tx as u64).wrapping_add(500).wrapping_add(((cy + dy) as u64).wrapping_mul(83)).wrapping_add((dx as u64).wrapping_mul(17)));
                    if v % 8 > 1 {
                        let c = cols[(v % 3) as usize];
                        set_px(buf, tx + dx, cy + dy, c);
                    }
                }
            }
        }
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

fn draw_fireplace(buf: &mut PixBuf, pw: usize, ph: usize) {
    let side_w   = (pw * 19 / 100).max(4);
    let mantel_h = (ph * 11 / 100).max(2);
    let hearth_y = ph.saturating_sub((ph * 9 / 100).max(2));

    let mortar   = Color::Rgb(108, 102, 96);
    let stone: (u8, u8, u8) = (118, 115, 108);
    let stone_dk = Color::Rgb(88,  85,  80);

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
                let c = if row_b == 0 || col_b == 0 {
                    mortar
                } else {
                    let v = hash(((row * 13 + (ppx - xs + off) / brick_w) as u64).wrapping_add(77));
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

    // Mantelpiece — stone shelf with a slight overhang shadow underneath
    for py in 0..mantel_h {
        for ppx in 0..pw {
            if py >= buf.len() || ppx >= buf[0].len() { continue; }
            let v = hash(((ppx / 4) as u64).wrapping_add((py / 2 * 97) as u64));
            buf[py][ppx] = Some(if py == mantel_h - 1 {
                stone_dk
            } else {
                Color::Rgb(
                    stone.0.saturating_add((v % 18) as u8),
                    stone.1.saturating_add(((v >> 4) % 14) as u8),
                    stone.2.saturating_add(((v >> 8) % 12) as u8),
                )

            });
        }
    }

    // Hearth floor — stone flags
    for py in hearth_y..ph {
        for ppx in 0..pw {
            if py >= buf.len() || ppx >= buf[0].len() { continue; }
            let v = hash(((ppx / 5) as u64).wrapping_add((py * 53) as u64));
            buf[py][ppx] = Some(Color::Rgb(
                78u8.saturating_add((v % 18) as u8),
                76u8.saturating_add(((v >> 4) % 14) as u8),
                72u8.saturating_add(((v >> 8) % 11) as u8),
            ));
        }
    }
}

// ── Landscape fill functions ──────────────────────────────────────────────────

fn fill_waves(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let t = tick as f64 * 0.12;
    let horizon = ph * 3 / 10;
    for py in 0..ph {
        for ppx in 0..pw {
            buf[py][ppx] = Some(if py < horizon {
                let frac = py as f64 / horizon as f64;
                Color::Rgb((8.0 + frac*18.0) as u8, (18.0 + frac*38.0) as u8, (55.0 + frac*75.0) as u8)
            } else {
                let wy = py - horizon;
                let wh = ph - horizon;
                let w1 = (ppx as f64 * 0.35 + t).sin() * 2.5;
                let w2 = (ppx as f64 * 0.18 - t * 0.65).sin() * 1.5;
                let surf = (wy as f64 - (w1 + w2)).abs();
                if surf < 1.3 { Color::Rgb(200, 230, 255) }
                else if surf < 2.5 { Color::Rgb(120, 185, 230) }
                else {
                    let depth = wy as f64 / wh as f64;
                    Color::Rgb(0, (90.0 - depth*50.0) as u8, (200.0 - depth*80.0) as u8)
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

fn fill_rain(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
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

fn fill_leaves(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
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

fn fill_stars(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
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
}

fn fill_fire(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let t        = tick as f64 * 0.13;
    let side_w   = (pw * 19 / 100).max(4);
    let mantel_h = (ph * 11 / 100).max(2);
    let hearth_y = ph.saturating_sub((ph * 9 / 100).max(2));
    let fire_x0  = side_w;
    let fire_x1  = pw.saturating_sub(side_w);
    let fire_pw  = fire_x1.saturating_sub(fire_x0);
    let fire_ph  = hearth_y.saturating_sub(mantel_h);

    // Dark room background
    for py in 0..ph { for ppx in 0..pw { buf[py][ppx] = Some(Color::Rgb(5, 4, 3)); } }

    // Fire in opening
    for ppx in fire_x0..fire_x1 {
        let local_x = ppx - fire_x0;
        let cx = (local_x as f64 - fire_pw as f64 / 2.0).abs() / (fire_pw as f64 / 2.0);
        let flicker = (ppx as f64 * 0.6 + t * 2.1).sin() * 0.13
                    + (ppx as f64 * 0.3 - t * 1.4).sin() * 0.08;
        let height = ((1.0 - cx * 0.55 + flicker).clamp(0.0, 1.0) * 0.85 * fire_ph as f64) as usize;
        let top    = hearth_y.saturating_sub(height);
        for py in mantel_h..hearth_y {
            buf[py][ppx] = Some(if py < top {
                let sd = top as isize - py as isize;
                if sd < 4 { let v = sd as u8 * 14; Color::Rgb(v / 2, v / 3, v / 3) }
                else { Color::Rgb(10, 6, 5) }
            } else {
                let f = (py - top) as f64 / height.max(1) as f64;
                if f < 0.25 { let v = f/0.25; Color::Rgb(255, (200.0*v+55.0) as u8, (80.0*(1.0-v)) as u8) }
                else if f < 0.65 { let v = (f-0.25)/0.40; Color::Rgb(255, (55.0*(1.0-v)) as u8, 0) }
                else { Color::Rgb(((1.0-(f-0.65)/0.35)*255.0) as u8, 0, 0) }
            });
        }
    }

    // Fireplace structure drawn on top
    draw_fireplace(buf, pw, ph);
}

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

    // Dark mid-ground fill — per-column so no rectangular edges
    let terrain_top_base = ph * 66 / 100;
    for ppx in 0..pw {
        let gy = ground_at(ppx);
        let xf = ppx as f64 / pw as f64;
        let tt_offset = ((xf * pi * 1.8).sin() * 0.5 + 0.5) * ph as f64 * 0.04;
        let tt = (terrain_top_base as f64 - tt_offset).max(0.0) as usize;
        for py in tt..gy {
            if py < buf.len() && ppx < buf[0].len() {
                let d = (py - tt) as f64 / (gy - tt).max(1) as f64;
                buf[py][ppx] = Some(Color::Rgb((5.0 + d*5.0) as u8, (7.0 + d*7.0) as u8, (16.0 + d*10.0) as u8));
            }
        }
    }

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

fn fill_aurora(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
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
                let bottom = aurora_h + (((ppx as f64 * 0.15 + t * 0.5 + b).sin() * 0.06) * ph as f64) as usize;
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

// ── Theme list ────────────────────────────────────────────────────────────────

type FillFn = fn(&mut PixBuf, usize, usize, u64);

struct Theme { fill: FillFn, color: Color }

const THEMES: &[Theme] = &[
    Theme { fill: fill_waves,  color: Color::Rgb(0,   120, 210) },
    Theme { fill: fill_rain,   color: Color::Rgb(80,  130, 210) },
    Theme { fill: fill_leaves, color: Color::Rgb(210, 95,  15)  },
    Theme { fill: fill_stars,  color: Color::Rgb(140, 140, 255) },
    Theme { fill: fill_fire,   color: Color::Rgb(255, 90,  0)   },
    Theme { fill: fill_aurora, color: Color::Rgb(20,  210, 150) },
];

// ── Public Animation struct ───────────────────────────────────────────────────

pub struct Animation {
    theme_idx: usize,
    pub render_mode: RenderMode,
    tick_count: u64,
}

impl Animation {
    pub fn new() -> Self {
        Self { theme_idx: 0, render_mode: RenderMode::Half, tick_count: 0 }
    }

    pub fn tick(&mut self) { self.tick_count += 1; }

    pub fn next_theme(&mut self) { self.theme_idx = (self.theme_idx + 1) % THEMES.len(); }
    pub fn prev_theme(&mut self) { self.theme_idx = (self.theme_idx + THEMES.len() - 1) % THEMES.len(); }
    pub fn next_mode(&mut self)  { self.render_mode = self.render_mode.next(); }
    pub fn prev_mode(&mut self)  { self.render_mode = self.render_mode.prev(); }

    pub fn theme_color(&self) -> Color { THEMES[self.theme_idx].color }

    pub fn render_lines(&self, _phase: &crate::timer::Phase, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
        if char_w == 0 || char_h == 0 { return vec![]; }
        let mode = self.render_mode;
        let theme = &THEMES[self.theme_idx];
        let (pw, ph) = (mode.pw(char_w), mode.ph(char_h));
        let mut buf = new_buf(pw, ph);
        (theme.fill)(&mut buf, pw, ph, self.tick_count);
        dispatch(mode, &buf, char_w, char_h)
    }
}

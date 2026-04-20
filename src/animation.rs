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
}

fn fill_rain(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64) {
    let ground_y = ph * 88 / 100;
    for py in 0..ph {
        for ppx in 0..pw {
            buf[py][ppx] = Some(if py >= ground_y {
                let f = (py - ground_y) as f64 / (ph - ground_y) as f64;
                Color::Rgb((15.0 + f*10.0) as u8, (28.0 + f*12.0) as u8, (15.0 + f*8.0) as u8)
            } else {
                let frac = py as f64 / ground_y as f64;
                Color::Rgb((8.0 + frac*6.0) as u8, (12.0 + frac*8.0) as u8, (22.0 + frac*12.0) as u8)
            });
        }
    }
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
    for py in ground_y..ph {
        for ppx in 0..pw {
            let ripple = (ppx as f64 * 0.4 + tick as f64 * 0.15).sin();
            if ripple > 0.7 { buf[py][ppx] = Some(Color::Rgb(30, 55, 80)); }
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
    let leaf_colors = [
        Color::Rgb(210, 65, 10), Color::Rgb(195, 130, 0),
        Color::Rgb(170, 75, 20), Color::Rgb(145, 95, 30), Color::Rgb(220, 160, 0),
    ];
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
                let (lx, ly) = (x + dx, y + dy);
                if lx >= 0 && lx < pw as isize && ly >= 0 && (ly as usize) < ground_y {
                    buf[ly as usize][lx as usize] = Some(col);
                }
            }
        }
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
    let t = tick as f64 * 0.13;
    for ppx in 0..pw {
        let cx = (ppx as f64 - pw as f64 / 2.0).abs() / (pw as f64 / 2.0);
        let flicker = (ppx as f64 * 0.6 + t * 2.1).sin() * 0.13
                    + (ppx as f64 * 0.3 - t * 1.4).sin() * 0.08;
        let height = ((1.0 - cx * 0.55 + flicker).clamp(0.0, 1.0) * 0.85 * ph as f64) as usize;
        let top = ph.saturating_sub(height);
        for py in 0..ph {
            buf[py][ppx] = Some(if py < top {
                let sd = top as isize - py as isize;
                if sd < 4 { let v = sd as u8 * 18; Color::Rgb(v/2, v/3, v/3) }
                else { Color::Rgb(0, 0, 0) }
            } else {
                let f = (py - top) as f64 / height.max(1) as f64;
                if f < 0.25 { let v = f/0.25; Color::Rgb(255, (200.0*v+55.0) as u8, (80.0*(1.0-v)) as u8) }
                else if f < 0.65 { let v = (f-0.25)/0.40; Color::Rgb(255, (55.0*(1.0-v)) as u8, 0) }
                else { Color::Rgb(((1.0-(f-0.65)/0.35)*255.0) as u8, 0, 0) }
            });
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

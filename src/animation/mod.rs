use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

mod waves;
mod rain;
mod leaves;
mod stars;
mod fire;
mod aurora;
mod blossom;
mod sunset;

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

// ── Theme list ────────────────────────────────────────────────────────────────

type FillFn = fn(&mut PixBuf, usize, usize, u64);

struct Theme { fill: FillFn, color: Color }

const THEMES: &[Theme] = &[
    Theme { fill: waves::fill_waves,   color: Color::Rgb(0,   120, 210) },
    Theme { fill: rain::fill_rain,     color: Color::Rgb(80,  130, 210) },
    Theme { fill: leaves::fill_leaves, color: Color::Rgb(210, 95,  15)  },
    Theme { fill: stars::fill_stars,   color: Color::Rgb(140, 140, 255) },
    Theme { fill: fire::fill_fire,     color: Color::Rgb(255, 90,  0)   },
    Theme { fill: aurora::fill_aurora, color: Color::Rgb(20,  210, 150) },
    Theme { fill: blossom::fill_blossom, color: Color::Rgb(245, 170, 195) },
    Theme { fill: sunset::fill_sunset, color: Color::Rgb(220, 90,  20)  },

];

// ── Public Animation struct ───────────────────────────────────────────────────

pub struct Animation {
    focus_theme: usize,
    break_theme: usize,
    pub render_mode: RenderMode,
    tick_count: u64,
}

impl Animation {
    pub fn new_with(focus_theme: usize, break_theme: usize, render_mode: RenderMode) -> Self {
        Self {
            focus_theme: focus_theme % THEMES.len(),
            break_theme: break_theme % THEMES.len(),
            render_mode,
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) { self.tick_count += 1; }

    pub fn active_theme(&self, phase: &crate::timer::Phase) -> usize {
        match phase {
            crate::timer::Phase::Work => self.focus_theme,
            _ => self.break_theme,
        }
    }

    pub fn next_theme(&mut self, phase: &crate::timer::Phase) {
        let idx = self.active_theme(phase);
        let next = (idx + 1) % THEMES.len();
        match phase {
            crate::timer::Phase::Work => self.focus_theme = next,
            _ => self.break_theme = next,
        }
    }

    pub fn prev_theme(&mut self, phase: &crate::timer::Phase) {
        let idx = self.active_theme(phase);
        let prev = (idx + THEMES.len() - 1) % THEMES.len();
        match phase {
            crate::timer::Phase::Work => self.focus_theme = prev,
            _ => self.break_theme = prev,
        }
    }

    pub fn next_mode(&mut self)  { self.render_mode = self.render_mode.next(); }
    pub fn prev_mode(&mut self)  { self.render_mode = self.render_mode.prev(); }

    pub fn theme_color(&self, phase: &crate::timer::Phase) -> Color {
        THEMES[self.active_theme(phase)].color
    }

    pub fn render_lines(&self, phase: &crate::timer::Phase, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
        if char_w == 0 || char_h == 0 { return vec![]; }
        let mode = self.render_mode;
        let theme = &THEMES[self.active_theme(phase)];
        let (pw, ph) = (mode.pw(char_w), mode.ph(char_h));
        let mut buf = new_buf(pw, ph);
        (theme.fill)(&mut buf, pw, ph, self.tick_count);
        dispatch(mode, &buf, char_w, char_h)
    }
}

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::timer::Phase;
use crate::video::VideoAnimation;

const TOMATO_FRAMES: &[&[&str]] = &[
    &[
        "    ██    ",
        "   ████   ",
        "  ██  ██  ",
        " ████████ ",
        " ██    ██ ",
        " ████████ ",
        "  ██████  ",
    ],
    &[
        "   ██     ",
        "   ████   ",
        "  ██  ██  ",
        " ████████ ",
        " ██    ██ ",
        " ████████ ",
        "  ██████  ",
    ],
];

const SPROUT_FRAMES: &[&[&str]] = &[
    &[
        "          ",
        "    ██    ",
        "   ████   ",
        "  ██  ██  ",
        "    ██    ",
        "    ██    ",
        "  ██████  ",
    ],
    &[
        "    ██    ",
        "   ████   ",
        "  ██████  ",
        "   ████   ",
        "    ██    ",
        "    ██    ",
        "  ██████  ",
    ],
];

const FLAME_FRAMES: &[&[&str]] = &[
    &[
        "   ████   ",
        "  ██████  ",
        " ████████ ",
        " ████████ ",
        " ████████ ",
        "  ██████  ",
        "   ████   ",
    ],
    &[
        "  ██████  ",
        " ████████ ",
        " ████████ ",
        " ████████ ",
        "  ██████  ",
        "   ████   ",
        "    ██    ",
    ],
];

enum Kind {
    Static {
        frame_index: usize,
        tick_count: u64,
        ticks_per_frame: u64,
    },
    Video(VideoAnimation),
}

pub struct Animation {
    kind: Kind,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            kind: Kind::Static {
                frame_index: 0,
                tick_count: 0,
                ticks_per_frame: 8,
            },
        }
    }

    pub fn from_video(path: &str, w: usize, h: usize) -> std::io::Result<Self> {
        let va = VideoAnimation::load(path, w, h)?;
        Ok(Self { kind: Kind::Video(va) })
    }

    pub fn is_video(&self) -> bool {
        matches!(self.kind, Kind::Video(_))
    }

    pub fn tick(&mut self) {
        if let Kind::Static { frame_index, tick_count, ticks_per_frame } = &mut self.kind {
            *tick_count += 1;
            if *tick_count >= *ticks_per_frame {
                *tick_count = 0;
                *frame_index = (*frame_index + 1) % 2;
            }
        }
    }

    pub fn on_running_changed(&mut self, running: bool) {
        if let Kind::Video(va) = &mut self.kind {
            if running { va.play() } else { va.pause() }
        }
    }

    /// For static mode: render ratatui lines.
    pub fn render_lines(&self, phase: &Phase, char_w: usize, char_h: usize) -> Vec<Line<'static>> {
        match &self.kind {
            Kind::Static { frame_index, .. } => render_static_lines(phase, *frame_index, char_h),
            Kind::Video(_) => {
                // Video rendered separately via sixel; return empty placeholder.
                let _ = (char_w, char_h);
                vec![]
            }
        }
    }

    /// KGP path: borrow current raw frame for on-the-fly encoding.
    pub fn current_frame(&self) -> Option<&crate::video::RgbFrame> {
        match &self.kind {
            Kind::Video(va) => Some(va.current_frame()),
            Kind::Static { .. } => None,
        }
    }

    /// Sixel path: lazily encode current frame (cached per frame index).
    pub fn current_sixel(&mut self) -> Option<&str> {
        match &mut self.kind {
            Kind::Video(va) => Some(va.current_sixel()),
            Kind::Static { .. } => None,
        }
    }
}

fn render_static_lines(phase: &Phase, frame_index: usize, char_h: usize) -> Vec<Line<'static>> {
    let frames = match phase {
        Phase::Work => TOMATO_FRAMES,
        Phase::ShortBreak => SPROUT_FRAMES,
        Phase::LongBreak => FLAME_FRAMES,
    };
    let frame = frames[frame_index];
    let color = phase_color(phase);

    let pad_top = char_h.saturating_sub(frame.len()) / 2;
    let mut lines: Vec<Line<'static>> = (0..pad_top).map(|_| Line::from("")).collect();
    for &row in frame {
        lines.push(Line::from(Span::styled(row, Style::default().fg(color))));
    }
    lines
}

fn phase_color(phase: &Phase) -> Color {
    match phase {
        Phase::Work => Color::Red,
        Phase::ShortBreak => Color::Green,
        Phase::LongBreak => Color::Cyan,
    }
}

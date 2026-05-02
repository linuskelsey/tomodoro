use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::animation::{Animation, RenderMode};
use crate::timer::{Phase, Timer, TimerConfig};

pub struct LabelState {
    pub text: String,
}

pub struct EditState {
    pub fields: [(u64, u64); 3],  // (hours, minutes) per field
    pub selected: usize,
    pub unit: usize,  // 0=hours, 1=minutes
    pub typing_buf: String,
}

impl EditState {
    pub fn from_config(cfg: &TimerConfig) -> Self {
        let to_hm = |s: u64| (s / 3600, (s % 3600) / 60);
        Self {
            fields: [to_hm(cfg.work_secs), to_hm(cfg.short_break_secs), to_hm(cfg.long_break_secs)],
            selected: 0,
            unit: 1,
            typing_buf: String::new(),
        }
    }

    pub fn to_config(&self) -> TimerConfig {
        let to_secs = |(h, m): (u64, u64)| (h * 60 + m).max(1) * 60;
        TimerConfig {
            work_secs: to_secs(self.fields[0]),
            short_break_secs: to_secs(self.fields[1]),
            long_break_secs: to_secs(self.fields[2]),
            ..TimerConfig::default()
        }
    }
}

pub fn draw(f: &mut Frame, timer: &Timer, anim: &Animation, show_help: bool, edit_state: Option<&EditState>, label_state: Option<&LabelState>, startup: bool, volume: f32, endless: bool, vol_flash: (bool, bool), task_label: Option<&str>) {
    let area = f.area();
    if endless {
        draw_animation(f, timer, anim, area);
        return;
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(f, timer, rows[0], volume, vol_flash, task_label);
    draw_animation(f, timer, anim, rows[1]);
    draw_progress(f, timer, anim, rows[2]);

    if show_help {
        draw_help(f, area);
    }
    if let Some(es) = edit_state {
        draw_edit(f, es, area, startup);
    }
    if let Some(ls) = label_state {
        draw_label_input(f, ls, area);
    }
}

fn draw_header(f: &mut Frame, timer: &Timer, area: Rect, volume: f32, vol_flash: (bool, bool), task_label: Option<&str>) {
    let color = phase_color(&timer.phase);
    let phase_str = match timer.phase {
        Phase::Work => "F",
        Phase::ShortBreak => "B",
        Phase::LongBreak => "LB",
    };
    let interval = timer.config.long_break_interval as usize;
    let filled = (timer.sessions_completed as usize) % interval;
    let dots: String = "●".repeat(filled) + &"○".repeat(interval - filled);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(7), Constraint::Fill(1)])
        .split(area);

    let dim = Color::Rgb(70, 70, 70);
    let bracket = |lit: bool| Style::default().fg(if lit { Color::White } else { dim });
    let left = Line::from(vec![
        Span::styled(phase_str, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  vol: {}%  ", (volume * 100.0).round() as u8), Style::default().fg(dim)),
        Span::styled("[", bracket(vol_flash.0)),
        Span::styled(" ", Style::default().fg(dim)),
        Span::styled("]", bracket(vol_flash.1)),
    ]);
    f.render_widget(Paragraph::new(left), cols[0]);
    f.render_widget(
        Paragraph::new(Span::styled(timer.format_remaining(), Style::default().fg(color).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center),
        cols[1],
    );
    let right = if let Some(label) = task_label {
        Line::from(vec![
            Span::styled(label.to_string(), Style::default().fg(Color::Rgb(160, 160, 160))),
            Span::styled("  ", Style::default()),
            Span::styled(dots, Style::default().fg(color)),
        ])
    } else {
        Line::from(Span::styled(dots, Style::default().fg(color)))
    };
    f.render_widget(Paragraph::new(right).alignment(Alignment::Right), cols[2]);
}

fn draw_animation(f: &mut Frame, timer: &Timer, anim: &Animation, area: Rect) {
    let lines = anim.render_lines(&timer.phase, area.width as usize, area.height as usize);
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn draw_progress(f: &mut Frame, timer: &Timer, anim: &Animation, area: Rect) {
    let hint = " ? for help";
    let hint_width = hint.len() as u16 + 1;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(hint_width)])
        .split(area);

    f.render_widget(
        Paragraph::new(Span::styled(hint, Style::default().fg(Color::Rgb(60, 60, 60)))),
        cols[1],
    );

    let area = cols[0];
    let width = area.width as usize;
    let progress = timer.progress();
    let filled_color = anim.theme_color(&timer.phase);
    let empty_color = Color::Rgb(35, 35, 35);

    let line = if anim.render_mode == RenderMode::Braille {
        // Braille bar: 2 pixels per char, dots on the top row (bits 0x01 left, 0x08 right)
        let total_px = width * 2;
        let filled_px = (progress * total_px as f64) as usize;
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut run = String::new();
        let mut in_filled = true;

        for px in (0..total_px).step_by(2) {
            let l = px < filled_px;
            let r = (px + 1) < filled_px;
            // row-0 braille: dot1=0x01 (left), dot4=0x08 (right)
            let mask = (l as u8) | ((r as u8) << 3);
            let ch = char::from_u32(0x2800 | mask as u32).unwrap_or(' ');
            let this_filled = l || r;
            if this_filled == in_filled {
                run.push(ch);
            } else {
                let color = if in_filled { filled_color } else { empty_color };
                spans.push(Span::styled(run.clone(), Style::default().fg(color)));
                run.clear();
                in_filled = this_filled;
                run.push(ch);
            }
        }
        if !run.is_empty() {
            let color = if in_filled { filled_color } else { empty_color };
            spans.push(Span::styled(run, Style::default().fg(color)));
        }
        Line::from(spans)
    } else {
        // Centered bar: ━ (heavy horizontal) filled, ─ (light horizontal) empty
        let filled = (progress * width as f64) as usize;
        let mut spans: Vec<Span<'static>> = Vec::new();
        if filled > 0 {
            spans.push(Span::styled("━".repeat(filled), Style::default().fg(filled_color)));
        }
        if filled < width {
            spans.push(Span::styled("─".repeat(width - filled), Style::default().fg(empty_color)));
        }
        Line::from(spans)
    };

    f.render_widget(Paragraph::new(line), area);
}

fn draw_edit(f: &mut Frame, es: &EditState, area: Rect, startup: bool) {
    let labels = ["Focus", "Short break", "Long break"];
    let w = 32u16;
    // interior: 1 top_hint + (2 collapsed + 3 expanded) fields + 1 bot_hint = 7; +2 borders = 9
    let h = 9u16;
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let popup = Rect { x, y, width: w.min(area.width), height: h.min(area.height) };

    let hint_dim  = Style::default().fg(Color::Rgb(60, 60, 60));
    let arrow_sty = Style::default().fg(Color::Yellow);
    let bot_hint  = if startup { "  Enter: start" } else { "  Enter: apply  Esc: cancel" };

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled("  Tab: field   ← →: h/m", hint_dim)),
    ];

    // indent to value column: 2 spaces + 14 label + 1 space = 17 chars
    let val_indent = format!("{:17}", "");

    for (i, label) in labels.iter().enumerate() {
        let selected = i == es.selected;
        let (hv, mv) = es.fields[i];

        if selected {
            let h_ch_up = if es.unit == 0 { '▲' } else { ' ' };
            let m_ch_up = if es.unit == 1 { '▲' } else { ' ' };
            let h_ch_dn = if es.unit == 0 { '▼' } else { ' ' };
            let m_ch_dn = if es.unit == 1 { '▼' } else { ' ' };

            // ▲ row: indent + h_arrow + 2 spaces + m_arrow
            lines.push(Line::from(vec![
                Span::raw(val_indent.clone()),
                Span::styled(h_ch_up.to_string(), arrow_sty),
                Span::raw("  "),
                Span::styled(m_ch_up.to_string(), arrow_sty),
            ]));

            // value row
            let h_sty = if es.unit == 0 {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let m_sty = if es.unit == 1 {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14} ", label), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:02}", hv), h_sty),
                Span::raw(":"),
                Span::styled(format!("{:02}", mv), m_sty),
            ]));

            // ▼ row
            lines.push(Line::from(vec![
                Span::raw(val_indent.clone()),
                Span::styled(h_ch_dn.to_string(), arrow_sty),
                Span::raw("  "),
                Span::styled(m_ch_dn.to_string(), arrow_sty),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14} ", label), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:02}:{:02}", hv, mv), Style::default().fg(Color::Gray)),
            ]));
        }
    }

    lines.push(Line::from(Span::styled(bot_hint, hint_dim)));

    let title = if startup { " tomodoro " } else { " edit timers " };

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)).title(title)),
        popup,
    );
}

fn draw_label_input(f: &mut Frame, ls: &LabelState, area: Rect) {
    let w = 40u16;
    let h = 3u16;
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let popup = Rect { x, y, width: w.min(area.width), height: h.min(area.height) };

    let max_chars = (w as usize).saturating_sub(4);
    let display: String = ls.text.chars().take(max_chars).collect();
    let cursor = "█";

    let line = Line::from(vec![
        Span::styled(format!("  {}", display), Style::default().fg(Color::White)),
        Span::styled(cursor, Style::default().fg(Color::Yellow)),
    ]);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)).title(" task label ").title_bottom(Line::from(Span::styled(" Enter: confirm  Esc: cancel ", Style::default().fg(Color::Rgb(60, 60, 60)))).right_aligned())),
        popup,
    );
}

fn draw_help(f: &mut Frame, area: Rect) {
    let rows: &[(&str, &str)] = &[
        ("space",  "pause / resume"),
        ("n",      "next phase"),
        ("r",      "restart phase"),
        ("e",      "edit timers"),
        ("t",      "set task label"),
        ("[  ]",   "volume down / up"),
        ("← →",   "cycle theme"),
        ("↑ ↓",   "cycle render mode"),
        ("q",      "quit"),
        ("?",      "close help"),
    ];

    let w = 32u16;
    let h = rows.len() as u16 + 2;
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let popup = Rect { x, y, width: w.min(area.width), height: h.min(area.height) };

    let lines: Vec<Line> = rows.iter().map(|(key, desc)| {
        Line::from(vec![
            Span::styled(format!("  {:<6}", key), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}", desc), Style::default().fg(Color::White)),
        ])
    }).collect();

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray))),
        popup,
    );
}

fn phase_color(phase: &Phase) -> Color {
    match phase {
        Phase::Work => Color::Red,
        Phase::ShortBreak => Color::Green,
        Phase::LongBreak => Color::Cyan,
    }
}

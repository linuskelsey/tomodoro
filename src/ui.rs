use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::{animation::Animation, timer::Timer};

/// Returns the inner rect of the animation area (for sixel overlay positioning).
pub fn draw(f: &mut Frame, timer: &Timer, anim: &Animation) -> Rect {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // status bar
            Constraint::Min(0),    // animation + time
        ])
        .split(area);

    draw_status_bar(f, timer, chunks[0]);
    draw_main(f, timer, anim, chunks[1])
}

fn draw_status_bar(f: &mut Frame, timer: &Timer, area: Rect) {
    let phase_color = phase_color(&timer.phase);

    let sessions_span = Span::styled(
        format!(" ◉ {}", timer.sessions_completed),
        Style::default().fg(Color::Yellow),
    );
    let phase_span = Span::styled(
        format!(" {} ", timer.phase.label()),
        Style::default()
            .fg(phase_color)
            .add_modifier(Modifier::BOLD),
    );
    let status_span = Span::styled(
        if timer.running { " ▶ " } else { " ⏸ " },
        Style::default().fg(Color::DarkGray),
    );

    let line = Line::from(vec![sessions_span, Span::raw(" │"), phase_span, Span::raw("│"), status_span]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(line)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(para, area);
}

fn draw_main(f: &mut Frame, timer: &Timer, anim: &Animation, area: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // animation
            Constraint::Length(3), // time + progress
        ])
        .split(area);

    let anim_inner = draw_animation(f, timer, anim, chunks[0]);
    draw_timer(f, timer, chunks[1]);
    anim_inner
}

/// Returns the inner rect so the caller can overlay sixel graphics there.
fn draw_animation(f: &mut Frame, timer: &Timer, anim: &Animation, area: Rect) -> Rect {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);

    if anim.is_video() {
        // Inner area left blank; sixel written after draw() returns.
        f.render_widget(block, area);
    } else {
        let char_w = inner.width as usize;
        let char_h = inner.height as usize;
        let frame_lines = anim.render_lines(&timer.phase, char_w, char_h);
        let para = Paragraph::new(frame_lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(para, area);
    }

    inner
}

fn draw_timer(f: &mut Frame, timer: &Timer, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Time display
    let time_para = Paragraph::new(Line::from(Span::styled(
        timer.format_remaining(),
        Style::default()
            .fg(phase_color(&timer.phase))
            .add_modifier(Modifier::BOLD),
    )))
    .block(
        Block::default()
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .alignment(Alignment::Center);

    f.render_widget(time_para, chunks[0]);

    // Progress bar
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(
            Style::default()
                .fg(phase_color(&timer.phase))
                .bg(Color::Black),
        )
        .ratio(timer.progress());

    f.render_widget(gauge, chunks[1]);
}

fn phase_color(phase: &crate::timer::Phase) -> Color {
    match phase {
        crate::timer::Phase::Work => Color::Red,
        crate::timer::Phase::ShortBreak => Color::Green,
        crate::timer::Phase::LongBreak => Color::Cyan,
    }
}

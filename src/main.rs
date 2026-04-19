mod animation;
mod kitty;
mod sixel;
mod timer;
mod ui;
mod video;

use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableFocusChange, EnableFocusChange, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use animation::Animation;
use timer::Timer;

const TICK_MS: u64 = 100;
const DEFAULT_VIDEO_W: usize = 296;
const DEFAULT_VIDEO_H: usize = 152;

#[derive(Clone, Copy)]
enum Renderer { Kitty, Sixel }

fn detect_renderer(override_flag: Option<&str>, in_tmux: bool) -> Renderer {
    if let Some(r) = override_flag {
        return match r.to_ascii_lowercase().as_str() {
            "kitty" | "kgp" => Renderer::Kitty,
            _ => Renderer::Sixel,
        };
    }
    let term = std::env::var("TERM").unwrap_or_default();
    let prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let ghostty_env = std::env::var("GHOSTTY_RESOURCES_DIR").is_ok();
    if ghostty_env
        || term.contains("kitty") || term.contains("ghostty")
        || prog.eq_ignore_ascii_case("kitty")
        || prog.eq_ignore_ascii_case("ghostty")
    {
        return Renderer::Kitty;
    }
    if in_tmux { Renderer::Kitty } else { Renderer::Sixel }
}

fn parse_size(s: &str) -> Option<(usize, usize)> {
    let (w, h) = s.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let video_path = args.get(1).filter(|a| !a.starts_with('-')).cloned();

    let size = args.iter().position(|a| a == "--size")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| parse_size(s))
        .unwrap_or((DEFAULT_VIDEO_W, DEFAULT_VIDEO_H));

    let in_tmux = std::env::var("TMUX").is_ok();
    let renderer_flag = args.iter().position(|a| a == "--renderer")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());
    let renderer = detect_renderer(renderer_flag, in_tmux);

    let anim = if let Some(path) = video_path {
        match Animation::from_video(&path, size.0, size.1) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Warning: could not load video ({}), using built-in animation", e);
                Animation::new()
            }
        }
    } else {
        Animation::new()
    };

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableFocusChange)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, anim, renderer, in_tmux);

    if matches!(renderer, Renderer::Kitty) {
        let _ = write_raw(&kitty::delete(in_tmux));
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableFocusChange, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut anim: Animation,
    renderer: Renderer,
    in_tmux: bool,
) -> io::Result<()> {
    let mut timer = Timer::new();
    let tick = Duration::from_millis(TICK_MS);
    let use_kgp = anim.is_video() && matches!(renderer, Renderer::Kitty);
    let mut focused = true;
    let mut anim_rect = Rect::default();

    loop {
        terminal.draw(|f| {
            anim_rect = ui::draw(f, &timer, &anim);
        })?;

        // Only render video graphics when this pane has focus.
        // tmux passthrough bypasses cursor translation, so rendering while
        // unfocused places the image at the active pane's cursor (bleed).
        if anim.is_video() && focused {
            render_video(&mut anim, renderer, in_tmux, anim_rect)?;
        }

        let deadline = Instant::now() + tick;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if event::poll(remaining)? {
                match event::read()? {
                    Event::Key(key) => {
                        focused = true;
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('q'), _)
                            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                            (KeyCode::Char(' '), _) => {
                                timer.toggle();
                                anim.on_running_changed(timer.running);
                            }
                            (KeyCode::Char('n'), _) => { timer.advance(); }
                            (KeyCode::Char('r'), _) => timer.reset(),
                            _ => {}
                        }
                    }
                    Event::FocusLost => {
                        focused = false;
                        if use_kgp {
                            write_raw(&kitty::delete(in_tmux))?;
                        }
                    }
                    Event::FocusGained => {
                        focused = true;
                        // Redraw immediately rather than waiting for next tick.
                        terminal.clear()?;
                        terminal.draw(|f| {
                            anim_rect = ui::draw(f, &timer, &anim);
                        })?;
                        if anim.is_video() {
                            render_video(&mut anim, renderer, in_tmux, anim_rect)?;
                        }
                    }
                    Event::Resize(_, _) => {
                        // Resize implies pane is active; also fires when moved.
                        focused = true;
                        if use_kgp {
                            write_raw(&kitty::delete(in_tmux))?;
                        }
                        terminal.clear()?;
                        terminal.draw(|f| {
                            anim_rect = ui::draw(f, &timer, &anim);
                        })?;
                        if anim.is_video() {
                            render_video(&mut anim, renderer, in_tmux, anim_rect)?;
                        }
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        if timer.running {
            anim.tick();
            if timer.is_finished() {
                timer.advance();
            }
        }
    }
}

fn render_video(
    anim: &mut Animation,
    renderer: Renderer,
    in_tmux: bool,
    rect: Rect,
) -> io::Result<()> {
    match renderer {
        Renderer::Kitty => {
            if let Some(frame) = anim.current_frame() {
                write_at(rect, &kitty::encode(frame, rect.width, rect.height, in_tmux))?;
            }
        }
        Renderer::Sixel => {
            if let Some(data) = anim.current_sixel() {
                write_at(rect, data)?;
            }
        }
    }
    Ok(())
}

fn write_at(rect: Rect, data: &str) -> io::Result<()> {
    // Batch cursor move + image data in one write so tmux sees them atomically.
    let cursor = format!("\x1b[{};{}H", rect.y + 1, rect.x + 1);
    let mut stdout = io::stdout();
    stdout.write_all(cursor.as_bytes())?;
    stdout.write_all(data.as_bytes())?;
    stdout.flush()
}

fn write_raw(data: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(data.as_bytes())?;
    stdout.flush()
}

mod animation;
mod config;
mod history;
mod timer;
mod ui;

use std::{
    io::{self, Cursor},
    sync::mpsc,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableFocusChange, EnableFocusChange, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use animation::{Animation, RenderMode};
use config::AppConfig;
use timer::{Phase, Timer, TimerConfig};
use ui::{EditState, LabelState};

const TICK_MS: u64 = 100;
const SOUND_FOCUS_END: &[u8] = include_bytes!("../sounds/complete.oga");
const SOUND_BEEP: &[u8] = include_bytes!("../sounds/dialog-information.oga");

fn audio_thread() -> mpsc::SyncSender<(&'static [u8], f32)> {
    let (tx, rx) = mpsc::sync_channel::<(&'static [u8], f32)>(8);
    std::thread::spawn(move || {
        let Ok((_stream, handle)) = rodio::OutputStream::try_default() else { return };
        for (bytes, volume) in rx {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                sink.set_volume(volume);
                if let Ok(source) = rodio::Decoder::new(Cursor::new(bytes)) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    });
    tx
}

fn play_immediate(bytes: &'static [u8], volume: f32) {
    std::thread::spawn(move || {
        let Ok((_stream, handle)) = rodio::OutputStream::try_default() else { return };
        if let Ok(sink) = rodio::Sink::try_new(&handle) {
            sink.set_volume(volume);
            if let Ok(source) = rodio::Decoder::new(Cursor::new(bytes)) {
                sink.append(source);
                sink.sleep_until_end();
            }
        }
    });
}

fn versions_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".local/share/tomodoro")
}

fn cmd_install(version: &str) -> io::Result<()> {
    let root = versions_dir().join(version);
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            &format!("tomodoro@{}", version),
            "--root",
            root.to_str().unwrap_or("."),
        ])
        .status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

fn cmd_list() -> io::Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let base = versions_dir();
    let mut versions: Vec<String> = if let Ok(entries) = std::fs::read_dir(&base) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect()
    } else {
        vec![]
    };
    versions.sort();
    println!("* {} (current)", current);
    for v in versions {
        println!("  {}", v);
    }
    Ok(())
}

fn cmd_use(version: &str) -> io::Result<()> {
    let binary = versions_dir().join(version).join("bin").join("tomodoro");
    if !binary.exists() {
        eprintln!("tomodoro {} not installed. Run: tomodoro install {}", version, version);
        std::process::exit(1);
    }
    let mut args_iter = std::env::args().skip(1).peekable();
    let mut forward: Vec<String> = Vec::new();
    while let Some(a) = args_iter.next() {
        if a == "--use" || a == "-u" {
            args_iter.next();
        } else {
            forward.push(a);
        }
    }
    use std::os::unix::process::CommandExt;
    Err(std::process::Command::new(&binary).args(&forward).exec())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "tomodoro {}\n\nUSAGE:\n    tomodoro [OPTIONS]\n    tomodoro <COMMAND>\n\nOPTIONS:\n    -h, --help               Print this help\n    -V, --version            Print version\n    -E, --endless            Endless animation mode (no timers, no sounds)\n    -u, --use <version>      Launch a specific installed version\n\nCOMMANDS:\n    install <version>        Install a version from crates.io\n    list                     List installed versions\n    history                  Show session history",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("install") => match args.get(2) {
            Some(v) => return cmd_install(v),
            None => { eprintln!("Usage: tomodoro install <version>"); return Ok(()); }
        },
        Some("list") => return cmd_list(),
        Some("history") => { history::print_history(); return Ok(()); },
        _ => {}
    }

    if let Some(pos) = args.iter().position(|a| a == "--use" || a == "-u") {
        match args.get(pos + 1) {
            Some(v) => return cmd_use(v),
            None => { eprintln!("Usage: tomodoro --use <version>"); return Ok(()); }
        }
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableFocusChange,
        );
        default_hook(info);
    }));

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableFocusChange)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let endless = args.iter().any(|a| a == "--endless" || a == "-E");
    let cfg = AppConfig::load();
    let result = run(&mut terminal, endless, cfg);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableFocusChange, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, endless: bool, cfg: AppConfig) -> io::Result<()> {
    let audio = audio_thread();
    let timer_cfg = TimerConfig {
        work_secs: cfg.focus * 60,
        short_break_secs: cfg.short_break * 60,
        long_break_secs: cfg.long_break * 60,
        long_break_interval: cfg.long_break_interval,
    };
    let render_mode = match cfg.render_mode.as_str() {
        "quarter" => RenderMode::Quarter,
        "braille" => RenderMode::Braille,
        _ => RenderMode::Half,
    };
    let focus_theme = cfg.focus_theme.unwrap_or(cfg.theme);
    let break_theme = cfg.break_theme.unwrap_or(cfg.theme);
    let countdown_beeps = cfg.countdown_beeps;
    let notifications = cfg.notifications;
    let mut timer = Timer::new(timer_cfg.clone());
    let mut anim = Animation::new_with(focus_theme, break_theme, render_mode);
    let mut volume: f32 = cfg.volume.clamp(0.0, 1.0);
    let mut vol_flash: Option<(bool, Instant)> = None;
    let mut last_beep_sec: Option<u64> = None;
    let mut ding_pending: u8 = 0;
    let mut beep_pending: u8 = 0;
    let mut show_help = false;
    let mut edit_state: Option<EditState> = if endless || cfg.auto_start { None } else { Some(EditState::from_config(&timer_cfg)) };
    let mut label_state: Option<LabelState> = None;
    let mut task_label: Option<String> = None;
    let mut startup = !endless && !cfg.auto_start;
    let tick = Duration::from_millis(TICK_MS);

    if endless {
        timer.toggle(); // start running so anim ticks
    }

    loop {
        terminal.draw(|f| {
            let fl = vol_flash.map_or((false, false), |(right, t)| {
                let lit = t.elapsed() < Duration::from_millis(200);
                (!right && lit, right && lit)
            });
            ui::draw(f, &timer, &anim, show_help, edit_state.as_ref(), label_state.as_ref(), startup, volume, endless, fl, task_label.as_deref());
        })?;

        if !endless {
            for _ in 0..ding_pending {
                play_immediate(SOUND_FOCUS_END, volume);
            }
            ding_pending = 0;
            for _ in 0..beep_pending {
                let _ = audio.try_send((SOUND_BEEP, volume));
            }
            beep_pending = 0;
        } else {
            ding_pending = 0;
            beep_pending = 0;
        }

        let deadline = Instant::now() + tick;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if event::poll(remaining)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if endless {
                            match (key.code, key.modifiers) {
                                (KeyCode::Char('q'), _)
                                | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                                | (KeyCode::Esc, _) => return Ok(()),
                                (KeyCode::Char(' '), _) => timer.toggle(),
                                (KeyCode::Right, _) => anim.next_theme(&timer.phase),
                                (KeyCode::Left, _) => anim.prev_theme(&timer.phase),
                                (KeyCode::Up, _) => anim.next_mode(),
                                (KeyCode::Down, _) => anim.prev_mode(),
                                _ => {}
                            }
                        } else if let Some(ref mut es) = edit_state {
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(()),
                                KeyCode::Esc if startup => return Ok(()),
                                KeyCode::Esc => {
                                    edit_state = None;
                                }
                                KeyCode::Enter => {
                                    let new_cfg = es.to_config();
                                    timer.apply_config(new_cfg);
                                    last_beep_sec = None;
                                    edit_state = None;
                                    startup = false;
                                }
                                KeyCode::Char(c) if c.is_ascii_digit() => {
                                    let max = if es.unit == 0 { 23u64 } else { 59u64 };
                                    if es.typing_buf.len() >= 2 { es.typing_buf.remove(0); }
                                    es.typing_buf.push(c);
                                    let val = es.typing_buf.parse::<u64>().unwrap_or(0).min(max);
                                    if es.unit == 0 { es.fields[es.selected].0 = val; }
                                    else { es.fields[es.selected].1 = val; }
                                }
                                KeyCode::Tab => {
                                    es.typing_buf.clear();
                                    es.selected = (es.selected + 1) % 3;
                                }
                                KeyCode::Left => { es.typing_buf.clear(); es.unit = 0; }
                                KeyCode::Right => { es.typing_buf.clear(); es.unit = 1; }
                                KeyCode::Up => {
                                    es.typing_buf.clear();
                                    if es.unit == 0 {
                                        es.fields[es.selected].0 = (es.fields[es.selected].0 + 1).min(23);
                                    } else {
                                        let m = &mut es.fields[es.selected].1;
                                        *m = if *m < 59 { *m + 1 } else { 0 };
                                    }
                                }
                                KeyCode::Down => {
                                    es.typing_buf.clear();
                                    if es.unit == 0 {
                                        let h = &mut es.fields[es.selected].0;
                                        if *h > 0 { *h -= 1; }
                                    } else {
                                        let m = &mut es.fields[es.selected].1;
                                        *m = if *m > 0 { *m - 1 } else { 59 };
                                    }
                                }
                                _ => {}
                            }
                        } else if let Some(ref mut ls) = label_state {
                            match key.code {
                                KeyCode::Enter => {
                                    let trimmed = ls.text.trim().to_string();
                                    task_label = if trimmed.is_empty() { None } else { Some(trimmed) };
                                    label_state = None;
                                }
                                KeyCode::Esc => { label_state = None; }
                                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(()),
                                KeyCode::Backspace => { ls.text.pop(); }
                                KeyCode::Char(c) => { ls.text.push(c); }
                                _ => {}
                            }
                        } else {
                            match (key.code, key.modifiers) {
                                (KeyCode::Char('q'), _)
                                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                                (KeyCode::Esc, _) if show_help => show_help = false,
                                (KeyCode::Esc, _) => return Ok(()),
                                (KeyCode::Char('?'), _) => show_help = !show_help,
                                _ if show_help => show_help = false,
                                (KeyCode::Char('e'), _) => {
                                    edit_state = Some(EditState::from_config(&timer.config));
                                }
                                (KeyCode::Char('t'), _) => {
                                    label_state = Some(LabelState { text: task_label.clone().unwrap_or_default() });
                                }
                                (KeyCode::Char(' '), _) => timer.toggle(),
                                (KeyCode::Char('n'), _) => {
                                    if timer.advance() { ding_pending += 1; }
                                    last_beep_sec = None;
                                }
                                (KeyCode::Char('r'), _) => timer.reset(),
                                (KeyCode::Char(']'), _) => {
                                    volume = ((volume * 10.0 + 1.0).round() / 10.0).min(1.0);
                                    vol_flash = Some((true, Instant::now()));
                                }
                                (KeyCode::Char('['), _) => {
                                    volume = ((volume * 10.0 - 1.0).round() / 10.0).max(0.0);
                                    vol_flash = Some((false, Instant::now()));
                                }
                                (KeyCode::Right, _) => anim.next_theme(&timer.phase),
                                (KeyCode::Left, _) => anim.prev_theme(&timer.phase),
                                (KeyCode::Up, _) => anim.next_mode(),
                                (KeyCode::Down, _) => anim.prev_mode(),
                                _ => {}
                            }
                        }
                    }
                    Event::Resize(_, _) => terminal.clear()?,
                    _ => {}
                }
            } else {
                break;
            }
        }

        if timer.running {
            anim.tick();

            if !endless {
                if matches!(timer.phase, Phase::ShortBreak | Phase::LongBreak) {
                    let rem = timer.remaining().as_secs();
                    if rem > 0 && rem <= countdown_beeps && last_beep_sec != Some(rem) {
                        last_beep_sec = Some(rem);
                        beep_pending += 1;
                    }
                }

                if timer.is_finished() {
                    if notifications {
                        let msg = match timer.phase {
                            Phase::Work => "Focus session complete! Time for a break.",
                            Phase::ShortBreak | Phase::LongBreak => "Break over. Back to work!",
                        };
                        let _ = std::process::Command::new("notify-send")
                            .args(["🍅 tomodoro", msg])
                            .spawn();
                    }
                    if timer.phase == Phase::Work {
                        let dur_mins = timer.config.work_secs / 60;
                        history::log_session(dur_mins, task_label.as_deref());
                    }
                    timer.advance();
                    last_beep_sec = None;
                    ding_pending += 1;
                }
            }
        }
    }
}

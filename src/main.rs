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

use rodio::Source;

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
const SOUND_FOCUS_END: &[u8] = include_bytes!("../sounds/effects/bell.oga");
const SOUND_BEEP: &[u8] = include_bytes!("../sounds/effects/beep.oga");
const AMBIENT_WAVES:   &[u8] = include_bytes!("../sounds/tracks/waves.ogg");
const AMBIENT_RAIN:    &[u8] = include_bytes!("../sounds/tracks/rain.ogg");
const AMBIENT_FOREST:  &[u8] = include_bytes!("../sounds/tracks/leaves.ogg");
const AMBIENT_SPACE:   &[u8] = include_bytes!("../sounds/tracks/stars.ogg");
const AMBIENT_FIRE:    &[u8] = include_bytes!("../sounds/tracks/fire.ogg");
const AMBIENT_AURORA:  &[u8] = include_bytes!("../sounds/tracks/aurora.ogg");
const AMBIENT_BLOSSOM: &[u8] = include_bytes!("../sounds/tracks/blossom.ogg");
const AMBIENT_SUNSET:  &[u8] = include_bytes!("../sounds/tracks/sunset.ogg");

const COMPLETION_BASH: &str = r#"_tomodoro_completions() {
    local cur prev words cword
    _init_completion || return
    local commands="install list history completions"
    local flags="--help --version --endless --use"
    case "$prev" in
        --use)
            return ;;
        install)
            return ;;
        completions)
            COMPREPLY=( $(compgen -W "bash zsh fish" -- "$cur") )
            return ;;
    esac
    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "$flags" -- "$cur") )
    else
        COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
    fi
}
complete -F _tomodoro_completions tomodoro
"#;

const COMPLETION_ZSH: &str = r#"#compdef tomodoro
_tomodoro() {
    local -a commands flags
    commands=(
        'install:Install a version from crates.io'
        'list:List installed versions'
        'history:Show session history'
        'completions:Print shell completion script'
    )
    flags=(
        '(-h --help)'{-h,--help}'[Print help]'
        '(-V --version)'{-V,--version}'[Print version]'
        '(-E --endless)'{-E,--endless}'[Endless animation mode]'
        '(-u --use)'{-u,--use}'[Launch a specific installed version]:version'
    )
    if (( CURRENT == 2 )); then
        _describe 'command' commands -- flags
    elif (( CURRENT == 3 )); then
        case "$words[2]" in
            completions) _values 'shell' bash zsh fish ;;
        esac
    fi
}
_tomodoro
"#;

const COMPLETION_FISH: &str = r#"complete -c tomodoro -f
complete -c tomodoro -s h -l help      -d 'Print help'
complete -c tomodoro -s V -l version   -d 'Print version'
complete -c tomodoro -s E -l endless   -d 'Endless animation mode'
complete -c tomodoro -s u -l use       -d 'Launch a specific installed version' -r
complete -c tomodoro -n '__fish_use_subcommand' -a install     -d 'Install a version from crates.io'
complete -c tomodoro -n '__fish_use_subcommand' -a list        -d 'List installed versions'
complete -c tomodoro -n '__fish_use_subcommand' -a history     -d 'Show session history'
complete -c tomodoro -n '__fish_use_subcommand' -a completions -d 'Print shell completion script'
complete -c tomodoro -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish'
"#;

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

enum AmbientCmd { Play(&'static [u8], f32), Volume(f32), Stop }

fn ambient_thread() -> mpsc::Sender<AmbientCmd> {
    let (tx, rx) = mpsc::channel::<AmbientCmd>();
    std::thread::spawn(move || {
        let Ok((_stream, handle)) = rodio::OutputStream::try_default() else { return };
        let mut sink: Option<rodio::Sink> = None;
        for cmd in rx {
            match cmd {
                AmbientCmd::Play(bytes, vol) => {
                    if let Some(s) = sink.take() { s.stop(); }
                    if let Ok(s) = rodio::Sink::try_new(&handle) {
                        s.set_volume(vol);
                        if let Ok(src) = rodio::Decoder::new(Cursor::new(bytes)) {
                            s.append(src.repeat_infinite());
                        }
                        sink = Some(s);
                    }
                }
                AmbientCmd::Volume(v) => { if let Some(ref s) = sink { s.set_volume(v); } }
                AmbientCmd::Stop => { if let Some(s) = sink.take() { s.stop(); } }
            }
        }
    });
    tx
}

fn ambient_for_theme(idx: usize) -> Option<&'static [u8]> {
    match idx {
        0 => Some(AMBIENT_WAVES),
        1 => Some(AMBIENT_RAIN),
        2 => Some(AMBIENT_FOREST),
        3 => Some(AMBIENT_SPACE),
        4 => Some(AMBIENT_FIRE),
        5 => Some(AMBIENT_AURORA),
        6 => Some(AMBIENT_BLOSSOM),
        7 => Some(AMBIENT_SUNSET),
        _ => None,
    }
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

fn version_check() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Some(latest) = fetch_latest_version() {
            if is_newer(&latest, env!("CARGO_PKG_VERSION")) {
                let _ = tx.send(latest);
            }
        }
    });
    rx
}

fn fetch_latest_version() -> Option<String> {
    let out = std::process::Command::new("cargo")
        .args(["search", "tomodoro", "--limit", "1"])
        .output()
        .ok()?;
    let stdout = String::from_utf8(out.stdout).ok()?;
    let line = stdout.lines().find(|l| l.starts_with("tomodoro "))?;
    Some(line.split('"').nth(1)?.to_string())
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let mut p = v.split('.');
    Some((p.next()?.parse().ok()?, p.next()?.parse().ok()?, p.next()?.parse().ok()?))
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

fn start_inhibit() -> Option<std::process::Child> {
    std::process::Command::new("systemd-inhibit")
        .args(["--what=sleep:idle", "--who=tomodoro", "--why=Focus session", "--mode=block", "sleep", "infinity"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()
}

fn sync_inhibit(inhibit: &mut Option<std::process::Child>, active: bool) {
    match (inhibit.is_some(), active) {
        (false, true)  => { *inhibit = start_inhibit(); }
        (true,  false) => { if let Some(mut c) = inhibit.take() { let _ = c.kill(); } }
        _ => {}
    }
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
            "tomodoro {}\n\nUSAGE:\n    tomodoro [OPTIONS]\n    tomodoro <COMMAND>\n\nOPTIONS:\n    -h, --help               Print this help\n    -V, --version            Print version\n    -E, --endless            Endless animation mode (no timers, no sounds)\n    -u, --use <version>      Launch a specific installed version\n\nCOMMANDS:\n    install <version>        Install a version from crates.io\n    list                     List installed versions\n    history                  Show session history\n    completions <shell>      Print shell completion script (bash, zsh, fish)",
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
        Some("completions") => match args.get(2).map(|s| s.as_str()) {
            Some("bash") => { print!("{}", COMPLETION_BASH); return Ok(()); }
            Some("zsh")  => { print!("{}", COMPLETION_ZSH);  return Ok(()); }
            Some("fish") => { print!("{}", COMPLETION_FISH); return Ok(()); }
            _ => {
                eprintln!("Usage: tomodoro completions <bash|zsh|fish>\n");
                eprintln!("Examples:");
                eprintln!("  bash:  tomodoro completions bash >> ~/.bash_completion");
                eprintln!("         echo 'source ~/.bash_completion' >> ~/.bashrc");
                eprintln!("  zsh:   tomodoro completions zsh > ~/.zfunc/_tomodoro");
                eprintln!("  fish:  tomodoro completions fish > ~/.config/fish/completions/tomodoro.fish");
                return Ok(());
            }
        },
        _ => {}
    }

    if let Some(pos) = args.iter().position(|a| a == "--use" || a == "-u") {
        match args.get(pos + 1) {
            Some(v) => return cmd_use(v),
            None => { eprintln!("Usage: tomodoro --use <version>"); return Ok(()); }
        }
    }

    let known_flags = ["--endless", "-E"];
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') && !known_flags.contains(&arg.as_str()) {
            eprintln!("unrecognised flag: {}\nRun 'tomodoro --help' for usage.", arg);
            return Ok(());
        } else if !arg.starts_with('-') {
            eprintln!("unrecognised command: {}\nRun 'tomodoro --help' for usage.", arg);
            return Ok(());
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
    let ambient = ambient_thread();
    let mut last_ambient: Option<usize> = None;
    let update_rx = if cfg.update_check { Some(version_check()) } else { None };
    let mut update_notice: Option<String> = None;
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
    let bar_mode_override: Option<RenderMode> = match cfg.bar_style.as_deref() {
        Some("braille") => Some(RenderMode::Braille),
        Some("quarter") => Some(RenderMode::Quarter),
        Some("half")    => Some(RenderMode::Half),
        _               => None,
    };
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
    let mut inhibit: Option<std::process::Child> = None;
    let tick = Duration::from_millis(TICK_MS);

    if endless {
        timer.toggle(); // start running so anim ticks
    }

    loop {
        if update_notice.is_none() {
            if let Some(ref rx) = update_rx {
                if let Ok(v) = rx.try_recv() { update_notice = Some(v); }
            }
        }
        terminal.draw(|f| {
            let fl = vol_flash.map_or((false, false), |(right, t)| {
                let lit = t.elapsed() < Duration::from_millis(200);
                (!right && lit, right && lit)
            });
            ui::draw(f, &timer, &anim, show_help, edit_state.as_ref(), label_state.as_ref(), startup, volume, endless, fl, task_label.as_deref(), update_notice.as_deref(), bar_mode_override);
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
                        update_notice = None;
                        if endless {
                            match (key.code, key.modifiers) {
                                (KeyCode::Char('q'), _)
                                | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                                | (KeyCode::Esc, _) => { sync_inhibit(&mut inhibit, false); return Ok(()); }
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
                                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                    sync_inhibit(&mut inhibit, false);
                                    return Ok(());
                                }
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
                                (KeyCode::Char(' '), _) => {
                                    timer.toggle();
                                    sync_inhibit(&mut inhibit, timer.running && timer.phase == Phase::Work);
                                }
                                (KeyCode::Char('n'), _) => {
                                    if timer.advance() { ding_pending += 1; }
                                    last_beep_sec = None;
                                    sync_inhibit(&mut inhibit, timer.running && timer.phase == Phase::Work);
                                }
                                (KeyCode::Char('r'), _) => {
                                    timer.reset();
                                    sync_inhibit(&mut inhibit, timer.running && timer.phase == Phase::Work);
                                }
                                (KeyCode::Char(']'), _) => {
                                    volume = ((volume * 10.0 + 1.0).round() / 10.0).min(1.0);
                                    vol_flash = Some((true, Instant::now()));
                                    let _ = ambient.send(AmbientCmd::Volume(volume));
                                }
                                (KeyCode::Char('['), _) => {
                                    volume = ((volume * 10.0 - 1.0).round() / 10.0).max(0.0);
                                    vol_flash = Some((false, Instant::now()));
                                    let _ = ambient.send(AmbientCmd::Volume(volume));
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

        let desired = if timer.running { Some(anim.active_theme(&timer.phase)) } else { None };
        if desired != last_ambient {
            last_ambient = desired;
            match desired.and_then(ambient_for_theme) {
                Some(bytes) => { let _ = ambient.send(AmbientCmd::Play(bytes, volume)); }
                None => { let _ = ambient.send(AmbientCmd::Stop); }
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
                            .args(["-h", "boolean:suppress-sound:true", "🍅 tomodoro", msg])
                            .spawn();
                    }
                    if timer.phase == Phase::Work {
                        let dur_mins = timer.config.work_secs / 60;
                        history::log_session(dur_mins, task_label.as_deref());
                    }
                    timer.advance();
                    last_beep_sec = None;
                    ding_pending += 1;
                    sync_inhibit(&mut inhibit, timer.running && timer.phase == Phase::Work);
                }
            }
        }
    }
}

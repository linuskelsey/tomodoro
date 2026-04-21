mod animation;
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

use animation::Animation;
use timer::{Phase, Timer, TimerConfig};
use ui::EditState;

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

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
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

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableFocusChange, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let audio = audio_thread();
    let mut timer = Timer::new(TimerConfig::default());
    let mut anim = Animation::new();
    let mut volume: f32 = 1.0;
    let mut last_beep_sec: Option<u64> = None;
    let mut ding_pending: u8 = 0;
    let mut beep_pending: u8 = 0;
    let mut show_help = false;
    let mut edit_state: Option<EditState> = Some(EditState::from_config(&TimerConfig::default()));
    let mut startup = true;
    let tick = Duration::from_millis(TICK_MS);

    loop {
        terminal.draw(|f| {
            ui::draw(f, &timer, &anim, show_help, edit_state.as_ref(), startup, volume);
        })?;

        for _ in 0..ding_pending {
            play_immediate(SOUND_FOCUS_END, volume);
        }
        ding_pending = 0;
        for _ in 0..beep_pending {
            let _ = audio.try_send((SOUND_BEEP, volume));
        }
        beep_pending = 0;

        let deadline = Instant::now() + tick;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if event::poll(remaining)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if let Some(ref mut es) = edit_state {
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(()),
                                KeyCode::Esc if !startup => {
                                    edit_state = None;
                                }
                                KeyCode::Enter => {
                                    let new_cfg = es.to_config();
                                    timer.apply_config(new_cfg);
                                    last_beep_sec = None;
                                    edit_state = None;
                                    startup = false;
                                }
                                KeyCode::Tab => {
                                    es.selected = (es.selected + 1) % 3;
                                }
                                KeyCode::Left => { es.unit = 0; }
                                KeyCode::Right => { es.unit = 1; }
                                KeyCode::Up => {
                                    if es.unit == 0 {
                                        es.fields[es.selected].0 = (es.fields[es.selected].0 + 1).min(23);
                                    } else {
                                        let m = &mut es.fields[es.selected].1;
                                        *m = if *m < 59 { *m + 1 } else { 0 };
                                    }
                                }
                                KeyCode::Down => {
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
                                (KeyCode::Char(' '), _) => timer.toggle(),
                                (KeyCode::Char('n'), _) => {
                                    let was_work = timer.phase == Phase::Work;
                                    timer.advance();
                                    last_beep_sec = None;
                                    if was_work { ding_pending += 1; }
                                }
                                (KeyCode::Char('r'), _) => timer.reset(),
                                (KeyCode::Char(']'), _) => volume = (volume + 0.1).min(1.0),
                                (KeyCode::Char('['), _) => volume = (volume - 0.1).max(0.0),
                                (KeyCode::Right, _) => anim.next_theme(),
                                (KeyCode::Left, _) => anim.prev_theme(),
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

            if matches!(timer.phase, Phase::ShortBreak | Phase::LongBreak) {
                let rem = timer.remaining().as_secs();
                if rem > 0 && rem <= 5 && last_beep_sec != Some(rem) {
                    last_beep_sec = Some(rem);
                    beep_pending += 1;
                }
            }

            if timer.is_finished() {
                timer.advance();
                last_beep_sec = None;
                ding_pending += 1;
            }
        }
    }
}

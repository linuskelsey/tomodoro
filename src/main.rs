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
use timer::{Phase, Timer};

const TICK_MS: u64 = 100;
const SOUND_FOCUS_END: &[u8] = include_bytes!("../sounds/complete.oga");
const SOUND_BEEP: &[u8] = include_bytes!("../sounds/dialog-information.oga");

fn audio_thread() -> mpsc::SyncSender<&'static [u8]> {
    let (tx, rx) = mpsc::sync_channel::<&'static [u8]>(8);
    std::thread::spawn(move || {
        let Ok((_stream, handle)) = rodio::OutputStream::try_default() else { return };
        for bytes in rx {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                if let Ok(source) = rodio::Decoder::new(Cursor::new(bytes)) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    });
    tx
}

fn main() -> io::Result<()> {
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
    let mut timer = Timer::new();
    let mut anim = Animation::new();
    let mut last_beep_sec: Option<u64> = None;
    let mut focus_end_pending: u8 = 0;
    let mut beep_pending: u8 = 0;
    let mut show_help = false;
    let tick = Duration::from_millis(TICK_MS);

    loop {
        terminal.draw(|f| {
            ui::draw(f, &timer, &anim, show_help);
        })?;

        for _ in 0..focus_end_pending {
            let _ = audio.try_send(SOUND_FOCUS_END);
        }
        focus_end_pending = 0;
        for _ in 0..beep_pending {
            let _ = audio.try_send(SOUND_BEEP);
        }
        beep_pending = 0;

        let deadline = Instant::now() + tick;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if event::poll(remaining)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                        (KeyCode::Char('?'), _) => show_help = !show_help,
                        (KeyCode::Esc, _) if show_help => show_help = false,
                        _ if show_help => show_help = false,
                        (KeyCode::Char(' '), _) => timer.toggle(),
                        (KeyCode::Char('n'), _) => {
                            let was_work = timer.phase == Phase::Work;
                            timer.advance();
                            last_beep_sec = None;
                            if was_work { focus_end_pending += 1; }
                        }
                        (KeyCode::Char('r'), _) => timer.reset(),
                        (KeyCode::Right, _) => anim.next_theme(),
                        (KeyCode::Left, _) => anim.prev_theme(),
                        (KeyCode::Up, _) => anim.next_mode(),
                        (KeyCode::Down, _) => anim.prev_mode(),
                        _ => {}
                    },
                    Event::Resize(_, _) => terminal.clear()?,
                    _ => {}
                }
            } else {
                break;
            }
        }

        if timer.running {
            anim.tick();

            // Countdown beeps: last 5 seconds of any break
            if matches!(timer.phase, Phase::ShortBreak | Phase::LongBreak) {
                let rem = timer.remaining().as_secs();
                if rem > 0 && rem <= 5 && last_beep_sec != Some(rem) {
                    last_beep_sec = Some(rem);
                    beep_pending += 1;
                }
            }

            if timer.is_finished() {
                let was_work = timer.phase == Phase::Work;
                timer.advance();
                last_beep_sec = None;
                if was_work { focus_end_pending += 1; }
            }
        }
    }
}

# Changelog

## [0.6.0] - 2026-05-05
### Added
- Mute — `m` key mutes and unmutes in both regular and endless mode; restores previous volume level on unmute
- Endless mode volume controls — `[`/`]` now work in endless mode; volume overlay appears top-left matching regular mode header position; persists for 500ms; shows `vol: muted` when muted
- Endless mode help — `?` opens a dedicated help overlay listing endless-specific controls
- Session history redesign — `tomodoro history` now shows a grouped table by day and task with start time, end time, and session count per row; summary stats at the top (avg session length, avg sessions per day, best day); defaults to last 20 rows; pass `--full` to show complete history
- Timer profiles — define named presets in config as `[profiles.name]` with `focus`, `short_break`, `long_break` fields; startup screen shows a profile picker when profiles are defined; selecting a profile auto-labels the session with the profile name
- `default_profile` config key — loads a named profile silently at startup; works with `auto_start = true` to skip the picker entirely
- Custom effect sounds — `bell_sound` and `beep_sound` config keys accept a path to any ogg, mp3, wav, or flac file; falls back to built-in sounds if not set or file missing
- Sounds folder — `~/.config/tomodoro/sounds/effects/` and `~/.config/tomodoro/sounds/tracks/` created automatically on first launch

### Changed
- Update check now uses the crates.io HTTP API via `curl` instead of `cargo search`; faster and no longer requires cargo on PATH
- Session history start time now correctly reflects when the session began rather than when it ended

### Fixed
- Install instructions now prominently surface the `libasound2-dev` requirement for Debian/Ubuntu/Mint users at the point of installation, not only in the Requirements section

---

## [0.5.2] - 2026-05-04
### Added
- Config validation — unrecognised keys and out-of-range values are caught on launch; bad entries are removed from the config file, reset to defaults, and reported via an in-app warning popup dismissible with any key
- Config migration — when a new config key is introduced in an update, it is automatically added to the user's existing config file as a commented default without touching any previously set values

### Fixed
- `bar_style` comment in generated config now shows `"half"` as the example value

---

## [0.5.1] - 2026-05-03
### Added
- `bar_style` config option — lock the progress bar to a specific style (`half`, `quarter`, `braille`) independent of the animation render mode
- Shell completions — `tomodoro completions <bash|zsh|fish>` prints a completion script for the given shell; pipe into your shell's completion directory to enable tab completion

### Fixed
- Desktop notifications no longer trigger a system notification sound; app bell and beeps are unaffected

---

## [0.5.0] - 2026-05-03
### Added
- Ambient audio — looping background track per scene; all 8 animations covered; tracks switch instantly on scene change; plays while timer is running
- Volume `[`/`]` keys now also control ambient level in real time
- Update check — notifies on startup if a newer version is available on crates.io; dismissible with any key; disable with `update_check = false` in config
- Seagull in waves scene — flies in, lands on the mast, preens, and departs

---

## [0.4.1] - 2026-05-02
### Added
- Systemd inhibit — blocks sleep and idle while a focus session is running; releases on pause, break, or quit
- Unknown flags and commands now print an error and point to `--help` instead of silently launching the app

### Fixed
- `history` missing from `--help` output

---

## [0.4.0] - 2026-05-02
### Added
- Config file — `~/.config/tomodoro/config.toml` auto-created on first launch with commented defaults; covers themes, durations, volume, auto-start, countdown beeps, notifications, and long break interval
- Task labeling — press `t` mid-session to name the current task; shown in header alongside session dots
- Session history — completed focus sessions saved to `~/.local/share/tomodoro/history.json`; view with `tomodoro history`
- Desktop notifications — optional `notify-send` alerts on phase end via `notifications = true` in config
- Per-phase themes — `focus_theme` and `break_theme` config keys; `←`/`→` now sets the theme for the current phase independently

---

## [0.3.4] - 2026-05-02
### Fixed
- Timing buffer patch
- Background tree trunks in autumn scene too thick in small windows — halved width, reduced taper

---

## [0.3.3] - 2026-04-24
### Added
- Blossom scene tweaks
- Animation refinements across scenes

---

## [0.3.2] - 2026-04-23
### Fixed
- Minor patch fixes

---

## [0.3.1] - 2026-04-22
### Added
- Endless mode — `tomodoro -E` / `--endless`; full-screen animation with no timer, sounds, or UI chrome; `←`/`→`/`↑`/`↓` cycle themes and render modes

---

## [0.3.0] - 2026-04-22
### Added
- 8 hand-crafted animated scenes: waves, rain, leaves, stars, fire, aurora, blossom, sunset
- 3 render modes: half-block, quarter-block, braille
- Bell sounds and countdown beeps
- CLI flags: `--help`, `--version`
- Version management: `tomodoro install <version>`, `list`, `--use <version>`

---

## [0.2.2] - 2026-04-21
### Added
- Volume control (`[`/`]` keys, shown in header)

---

## [0.2.0] - 2026-04-20
### Added
- Custom timer durations on startup screen and mid-session with `e`
- Tab/arrow/digit input for timer fields

---

## [0.1.1] - 2026-04-20
### Fixed
- Sound patch

---

## [0.1.0] - 2026-04-20
Initial release — terminal Pomodoro timer with session tracking, progress bar, phase indicators, and bell sounds.

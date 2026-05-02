# Changelog

## [0.4.1] - unreleased
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

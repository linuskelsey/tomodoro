# Roadmap

## Planned

### Major
- **TUI redesign** — distinct tabbed sections navigable with Tab/Shift-Tab; animation tab houses the clock and rendered scene; audio tab styled like a radio panel showing current stream/track, volume, and controls; design inspired by bluetui/impala; this is the largest structural change and will ship alongside the new animation set and internet radio
- **New animation set** — full redraw of all themes from scratch; two-tone (black + terminal foreground colour); higher resolution than current; dithered/ordered-halftone style for atmospheric scenes, chunky pixel art for character/creature scenes; both styles suit KGP/sixel rendering cleanly
- **Package manager distribution** — publish to apt, pacman (AUR), and Homebrew; resolves system dependency issues (e.g. libasound2-dev on Debian-based distros) transparently for users; no feature flags or manual steps required
- **Spotify integration** — connect Spotify account via OAuth; auto play/pause on session start/end; user selects a playlist to shuffle during focus sessions
- **Internet radio channels** — built-in curated list of stream URLs (NTS, Soma FM, etc.); selectable from config or a TUI picker; streams play during focus sessions via an HTTP audio backend; user can add custom stream URLs in config
- **Custom animations** — import user-made pixel art as animation frames; define frame sequences in config pointing at files (e.g. PNG strips or Aseprite exports); support common terminal pixel art editors (timg-compatible, pixterm); frames rendered via the existing sixel/kitty path
- **Timer profiles + startup menu** — named presets in config (e.g. `[profiles.deep]`); startup screen shows a profile picker with a "custom" option leading to the current time-edit screen
- **Custom sound paths** — config keys to point bell and ambient tracks to user-provided files; allows full replacement of embedded audio

### Minor
- **Detail scaling** — different levels of scene detail for different terminal pane sizes
- **Fortune popup** — call `fortune` at end of each focus session; overlay popup dismissible with `q`/`Esc`
- **Daily focus goal** — set a target focus time per day in config; track progress and show in header or end-of-day summary

### Patch
- **XDG_CONFIG_HOME support** — respect `$XDG_CONFIG_HOME` for config path instead of hardcoding `~/.config/tomodoro`; affects users with non-standard XDG setups
- **Update check via crates.io API** — replace `cargo search` (slow, requires cargo on PATH) with a direct HTTP call to the crates.io API using `curl`; faster and more reliable
- **Endless mode volume display** — show volume level feedback when adjusting with `[`/`]` in endless mode; currently keys work but nothing is shown
- **Timestamped history output** — `tomodoro history` currently shows only totals and task breakdown; surface per-session rows with timestamp, duration, and label so users can see when each session occurred

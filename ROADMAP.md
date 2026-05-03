# Roadmap

## Planned

### Major
- **New animation set** — full redraw of all themes from scratch; two-tone (black + terminal foreground colour); higher resolution than current; dithered/ordered-halftone style for atmospheric scenes, chunky pixel art for character/creature scenes; both styles suit KGP/sixel rendering cleanly
- **Spotify integration** — connect Spotify account via OAuth; auto play/pause on session start/end; user selects a playlist to shuffle during focus sessions
- **Timer profiles + startup menu** — named presets in config (e.g. `[profiles.deep]`); startup screen shows a profile picker with a "custom" option leading to the current time-edit screen
- **Custom sound paths** — config keys to point bell and ambient tracks to user-provided files; allows full replacement of embedded audio

### Minor
- **Detail scaling** — different levels of scene detail for different terminal pane sizes
- **Fortune popup** — call `fortune` at end of each focus session; overlay popup dismissible with `q`/`Esc`
- **Daily focus goal** — set a target focus time per day in config; track progress and show in header or end-of-day summary

### Patch
- **alsa-sys Ubuntu/Mint fix** — resolve build failure on Debian-based distros missing `libasound2-dev`; likely optional audio feature flag
- **Config validation warnings** — print a warning on launch when config has unknown keys or out-of-range values instead of silently ignoring them

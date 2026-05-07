# Roadmap

## >= 1.0.0
- **TUI redesign** — distinct tabbed sections navigable with Tab/Shift-Tab; animation tab houses the clock and rendered scene; audio tab styled like a radio panel showing current stream/track, volume, and controls; design inspired by bluetui/impala; this is the largest structural change and will ship alongside the new animation set and internet radio
- **New animation set** — full redraw of all themes from scratch; two-tone (black + terminal foreground colour); higher resolution than current; dithered/ordered-halftone style for atmospheric scenes, chunky pixel art for character/creature scenes; both styles suit KGP/sixel rendering cleanly
- **Package manager distribution** — publish to apt, pacman (AUR), and Homebrew; resolves system dependency issues (e.g. libasound2-dev on Debian-based distros) transparently for users; no feature flags or manual steps required
- **Internet radio channels** — built-in curated list of stream URLs (NTS, Soma FM, etc.); selectable from config or a TUI picker; streams play during focus sessions via an HTTP audio backend; user can add custom stream URLs in config
- **Custom animations** — import user-made pixel art as animation frames; define frame sequences in config pointing at files (e.g. PNG strips or Aseprite exports); support common terminal pixel art editors (timg-compatible, pixterm); frames rendered via the existing sixel/kitty path
- **Spotify integration** — connect Spotify account via OAuth; auto play/pause on session start/end; user selects a playlist to shuffle during focus sessions

## < 1.0

### Minor
- **Custom ambient tracks** — in-app audio file selector to assign user-provided tracks to themes; files placed in `~/.config/tomodoro/sounds/tracks/`; config stores assignments per theme
- **Detail scaling** — different levels of scene detail for different terminal pane sizes
- **Fortune popup** — call `fortune` at end of each focus session; overlay popup dismissible with `q`/`Esc`
- **Daily focus goal** — set a target focus time per day in config; track progress and show in header or end-of-day summary

### Patch
- **Session duration in history** — `tomodoro history` currently shows start time, end time, and session count but not the focus duration; add a duration column (or incorporate it into the existing view) so users can see how long each session was


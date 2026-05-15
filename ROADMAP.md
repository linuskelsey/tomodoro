# Roadmap

## Before 1.0 dev (ships to both classic and tomodoro)
- **Terminal palette colours** — query the terminal's actual color palette at startup via OSC 4 escape sequences; use the resolved RGB values of color1, color2, color4 as the default focus, short break, and long break colours; falls back to ANSI named colours on terminals that don't respond; requires no user config
- **Popup keypress lag** — what's new, fortune, and other popups share the 1s idle tick used for paused-timer state; active UI states (any open popup, edit, label, profile picker) should keep the 100ms tick so keystrokes feel instant
- **Fortune popup background bleed** — animation frames from the break theme (e.g. blossom) render through the fortune overlay; popup likely missing a `Clear` widget render before drawing its content; may be triggered specifically when a different break theme is set in config
- **Waybar widget latency** — switching from `bar_signal` to `interval: 1` polling introduced up to 1s update lag; ideal fix is an `inotifywait`-based long-lived exec script in waybar that watches `/tmp/tomodoro.json` for writes (instant, zero idle CPU, no signals or process spawning from tomodoro side, requires `inotify-tools`); intermediate option is `interval: 1` + `bar_signal` combined so signal gives instant update and polling is the fallback

## tomodoro-classic
- **Custom ambient tracks** — in-app audio file selector to assign user-provided tracks to themes; files placed in `~/.config/tomodoro/sounds/tracks/`; config stores assignments per theme
- **Detail scaling** — different levels of scene detail for different terminal pane sizes

## tomodoro >= 1.0
- **TUI redesign** — distinct tabbed sections navigable with Tab/Shift-Tab; animation tab houses the clock and rendered scene; audio tab styled like a radio panel showing current stream/track, volume, and controls; design inspired by bluetui/impala; this is the largest structural change and will ship alongside the new animation set and internet radio
- **New animation set** — full redraw of all themes from scratch; two-tone (black + terminal foreground colour); higher resolution than current; dithered/ordered-halftone style for atmospheric scenes, chunky pixel art for character/creature scenes; both styles suit KGP/sixel rendering cleanly
- **Documentation site** — GitHub Pages site for user-facing docs; covers install, config reference, bar integration, profiles, and theming; replaces the long README as the primary reference
- **Package manager distribution** — publish to apt, pacman (AUR), and Homebrew; resolves system dependency issues (e.g. libasound2-dev on Debian-based distros) transparently for users; no feature flags or manual steps required
- **Internet radio channels** — built-in curated list of stream URLs (NTS, Soma FM, etc.); selectable from config or a TUI picker; streams play during focus sessions via an HTTP audio backend; user can add custom stream URLs in config
- **Custom animations** — import user-made pixel art as animation frames; define frame sequences in config pointing at files (e.g. PNG strips or Aseprite exports); support common terminal pixel art editors (timg-compatible, pixterm); frames rendered via the existing sixel/kitty path
- **Spotify integration** — connect Spotify account via OAuth; auto play/pause on session start/end; user selects a playlist to shuffle during focus sessions
- **Command mode** — `:` opens a vim-style command prompt; supports commands like `:focus 50`, `:theme aurora`, `:vol 0.8`, `:skip`, `:pause`, `:label my task`; tab completion for command names and values; history navigable with `↑`/`↓`; offers a discoverable alternative to config edits and keybinds

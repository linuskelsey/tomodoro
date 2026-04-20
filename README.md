# tomodoro

A terminal Pomodoro timer with animated backgrounds.

![demo](demo.gif)

![Rust](https://img.shields.io/badge/rust-stable-orange)

## Install

```sh
cargo install tomodoro
```

Requires Rust — install via [rustup](https://rustup.rs) if you don't have it.

## Usage

On launch, a setup screen lets you choose your focus and break durations before the timer starts. Press `Esc` to use the defaults (25 / 5 / 15 min).

| Key | Action |
|-----|--------|
| `Space` | Start / pause |
| `n` | Skip to next phase |
| `r` | Restart current phase |
| `e` | Edit timer durations |
| `←` / `→` | Cycle animation themes |
| `↑` / `↓` | Cycle render modes (Half → Quarter → Braille) |
| `?` | Toggle help overlay |
| `q` / `Ctrl+C` | Quit |

## Features

- **Custom durations** — set focus, short break, and long break times on startup or mid-session with `e`
- **Session tracker** — dots in the top-right show progress toward a long break (every 4 sessions)
- **6 animated themes** — waves, rain, falling leaves, stars, fire, aurora; rendered with Unicode half-blocks, quarter-blocks, or braille characters
- **3 render modes** — increasing pixel density per terminal cell (half → quarter → braille)
- **Coloured progress bar** — matches the current theme; uses braille dots in braille mode
- **Bell sounds** — single bell when a focus session ends; countdown beeps for the last 5 seconds of a break
- **Phase indicators** — `F` (focus), `B` (short break), `LB` (long break)

## Requirements

- A terminal with true colour and Unicode support (Ghostty, Kitty, WezTerm, etc.)

## Contributing

Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

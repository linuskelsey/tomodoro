# tomodoro

A terminal Pomodoro timer with animated backgrounds.

![Rust](https://img.shields.io/badge/rust-stable-orange)

## Install

```sh
cargo install tomodoro
```

Requires Rust — install via [rustup](https://rustup.rs) if you don't have it.

To install from source:

```sh
git clone https://github.com/linuskelsey/tomodoro
cd tomodoro
cargo install --path .
```

## Usage

| Key | Action |
|-----|--------|
| `Space` | Start / pause |
| `n` | Skip to next phase |
| `r` | Restart current phase |
| `←` / `→` | Cycle animation themes |
| `↑` / `↓` | Cycle render modes (Half → Quarter → Braille) |
| `q` / `Ctrl+C` | Quit |
| `?` | Toggle help overlay |

## Features

- **25 min focus / 5 min break / 15 min long break** — standard Pomodoro intervals
- **Session tracker** — dots in the top-right show progress toward a long break (every 4 sessions)
- **6 animated themes** — waves, rain, falling leaves, stars, fire, aurora; rendered with Unicode half-blocks, quarter-blocks, or braille characters
- **3 render modes** — increasing pixel density per terminal cell (half → quarter → braille)
- **Coloured progress bar** — matches the current theme; uses braille dots in braille mode
- **Bell sounds** — single bell when a focus session ends; countdown beeps for the last 5 seconds of a break
- **Phase indicators** — `F` (focus), `B` (short break), `LB` (long break)

## Requirements

- A terminal with true colour and Unicode support (Ghostty, Kitty, WezTerm, etc.)

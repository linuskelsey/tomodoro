# Contributing to tomodoro

## Setup

```sh
git clone https://github.com/linuskelsey/tomodoro
cd tomodoro
cargo build
```

Run locally with:

```sh
cargo run
```

## Linux (Debian / Ubuntu / Mint)

`alsa-sys` requires ALSA development headers:

```sh
sudo apt install libasound2-dev
```

## Project structure

| File | Purpose |
|------|---------|
| `src/main.rs` | Event loop, input handling, app state |
| `src/timer.rs` | Timer state machine and config |
| `src/ui.rs` | All ratatui rendering, edit popup |
| `src/animation.rs` | Themes, render modes, frame ticking |

## Guidelines

- Keep PRs focused — one feature or fix per PR
- Run `cargo clippy` and `cargo fmt` before submitting
- Test in a true-colour terminal (Ghostty, Kitty, WezTerm)

## Planned features

See [ROADMAP.md](ROADMAP.md) for upcoming features open for contribution.

## Reporting issues

Open an issue on [GitHub](https://github.com/linuskelsey/tomodoro/issues).

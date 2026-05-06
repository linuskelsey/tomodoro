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
| `src/animation/mod.rs` | Pixel buffer, render modes (half/quarter/braille), theme list, `Animation` struct |
| `src/animation/waves.rs` | Waves scene |
| `src/animation/rain.rs` | Rain scene |
| `src/animation/leaves.rs` | Autumn leaves scene |
| `src/animation/stars.rs` | Starfield scene |
| `src/animation/fire.rs` | Fireplace scene |
| `src/animation/aurora.rs` | Aurora borealis scene |
| `src/animation/blossom.rs` | Cherry blossom scene |
| `src/animation/sunset.rs` | Sunset scene |

### Adding a new animation theme

1. Create `src/animation/<name>.rs` with a `pub(super) fn fill_<name>(buf: &mut PixBuf, pw: usize, ph: usize, tick: u64)` function
2. Declare it in `src/animation/mod.rs` with `mod <name>;`
3. Add an entry to the `THEMES` array in `mod.rs`: `Theme { fill: <name>::fill_<name>, color: Color::Rgb(...) }`

## Checklist before submitting a PR

- [ ] **Help screen updated** — if you added, removed, or rebound a key, update `draw_help` (and `draw_endless_help` if applicable) in `src/ui.rs`
- [ ] **Docs updated** — update `CHANGELOG.md` with a summary of the change; update `README.md` and `ROADMAP.md` if the feature or its scope changed
- [ ] **Version bumped** — increment the version in `Cargo.toml` following semver (patch for fixes, minor for new features, major for breaking changes)
- [ ] **PR focused** — one feature or fix per PR
- [ ] `cargo clippy` and `cargo fmt` pass
- [ ] Tested in a true-colour terminal (Ghostty, Kitty, WezTerm)

## Planned features

See [ROADMAP.md](ROADMAP.md) for upcoming features open for contribution.

## Reporting issues

Open an issue on [GitHub](https://github.com/linuskelsey/tomodoro/issues).

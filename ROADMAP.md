# Roadmap

## Planned

### Major
- **New animation set** — full redraw of all themes from scratch; two-tone (black + terminal foreground colour); higher resolution than current; dithered/ordered-halftone style for atmospheric scenes, chunky pixel art for character/creature scenes; both styles suit KGP/sixel rendering cleanly
- **Music / ambience** — background audio during sessions; looping ambient tracks per phase
  - Fire crackling loop for fire animation

### Minor
- **Session history persistence** — save completed sessions to `~/.local/share/tomodoro/history.json`; show daily counts, streaks, total hours
- **Task labeling** — type task name before/during session; shown in header; logged to history
- **Detail scaling** — different levels of scene detail for different terminal pane sizes
- **Fortune popup** — call `fortune` at end of each focus session; overlay popup dismissible with `q`/`Esc`
- **Systemd inhibit during work** — call `systemd-inhibit` to block sleep/screensaver while work session active; release on break

### Patch
- **alsa-sys Ubuntu/Mint fix** — resolve build failure on Debian-based distros missing `libasound2-dev`; likely optional audio feature flag


## Done

- Config file — `~/.config/tomodoro/config.toml`; auto-created on first launch with commented defaults; keys: `theme`, `render_mode`, `focus_theme`, `break_theme`, `focus`, `short_break`, `long_break`, `volume`, `long_break_interval`, `auto_start`, `countdown_beeps`, `notifications`
- Desktop notifications — `notify-send` on phase end; toggled via `notifications` in config

- Endless mode — `tomodoro -E` / `tomodoro --endless`; full-screen animation, no timer/sounds; space pauses, ←/→/↑/↓ cycle themes and render modes
- Version management — `tomodoro install <version>`, `tomodoro list`, `tomodoro --use <version>` to run older crates.io releases alongside current
- CLI flags — `--help` / `-h` prints usage; `--version` / `-V` prints version
- Custom timer durations (startup screen + mid-session edit)
- Timer input — Tab cycles fields, ←/→ selects h/m, ↑/↓ edits value, digits type directly; Esc quits startup or cancels edit
- Volume control (`[`/`]` keys, displayed in header)
- 8 animated themes, 3 render modes
- Bell sounds + countdown beeps
- Session tracker
- AI-crafted animation scenes:
  - Rain — medieval skyline with half-timbered halls, stone towers, spires, plant pots, mountains, cobblestone floor, puddle ripples
  - Leaves — full maple trunk, shimenawa + shide charms, background autumn trees, falling leaves
  - Waves — rocking boat, multi-layer wave body, cross-chop and specular glints
  - Fire — crackling fireplace, iron grate, wooden logs, oak mantelpiece, burgundy carpet
  - Stars — scrolling parallax starfield with sporadic UFO
  - Aurora — borealis bands with organic lower edges over snowy rolling hills and mountains
  - Blossom — cherry blossom orchard beneath Mount Fuji viewed from a Japanese-style balcony; falling petals, shoji frame, tea setup
  - Sunset — mountain range reflected in a lake at dusk; purple-to-orange sky, undulating grass edge, specular shimmer

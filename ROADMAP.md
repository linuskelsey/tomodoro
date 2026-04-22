# Roadmap

## Planned

- **alsa-sys Ubuntu/Mint fix** — resolve build failure on Debian-based distros missing `libasound2-dev`; likely optional audio feature flag
- **Music / ambience** — background audio during sessions; looping ambient tracks per phase
  - Fire crackling loop for fire animation
- **Detail scaling** — different levels of scene detail for different terminal pane sizes
- **Endless mode** — animation plays indefinitely with no timer; pure ambient display

## Done

- Version management — `tomodoro install <version>`, `tomodoro list`, `tomodoro --use <version>` to run older crates.io releases alongside current
- Custom timer durations (startup screen + mid-session edit)
- Timer arrow control — Tab cycles fields, ←/→ selects h/m, ↑/↓ edits value
- Volume control (`[`/`]` keys, displayed in header)
- 6 animated themes, 3 render modes
- Bell sounds + countdown beeps
- Session tracker
- Hand-crafted animation scenes:
  - Rain — medieval skyline, mountains, lamppost, cobblestone floor, puddle ripples
  - Leaves — full maple trunk, shimenawa + shide charms, background autumn trees, falling leaves
  - Waves — rocking boat, multi-layer wave body, cross-chop and specular glints
  - Fire — crackling fireplace with depth shadow, sparks, stone mantelpiece
  - Stars — scrolling parallax starfield with sporadic UFO
  - Aurora — borealis bands over snowy rolling hills and mountains

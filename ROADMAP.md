# Roadmap

## Planned

### Major
- **New animation set** — full redraw of all themes from scratch; two-tone (black + terminal foreground colour); higher resolution than current; dithered/ordered-halftone style for atmospheric scenes, chunky pixel art for character/creature scenes; both styles suit KGP/sixel rendering cleanly
- **Music / ambience** — background audio during sessions; looping ambient tracks per phase
  - Fire crackling loop for fire animation

### Minor
- **Detail scaling** — different levels of scene detail for different terminal pane sizes
- **Fortune popup** — call `fortune` at end of each focus session; overlay popup dismissible with `q`/`Esc`

### Patch
- **alsa-sys Ubuntu/Mint fix** — resolve build failure on Debian-based distros missing `libasound2-dev`; likely optional audio feature flag

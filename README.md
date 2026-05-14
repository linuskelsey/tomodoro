# tomodoro

A terminal Pomodoro timer with animated backgrounds.

![demo](demo.gif)

![Rust](https://img.shields.io/badge/rust-stable-orange)
[![crates.io](https://img.shields.io/crates/v/tomodoro)](https://crates.io/crates/tomodoro)

## Install

```sh
cargo install tomodoro
```

Requires Rust — install via [rustup](https://rustup.rs) if you don't have it.

**Debian / Ubuntu / Mint:** audio requires `libasound2-dev`. If the build fails, install it first:

```sh
sudo apt install libasound2-dev
```

## Usage

On launch, if you have profiles defined in config a picker lets you choose one — or select Custom to set durations manually. Use `Tab` / `Shift+Tab` to move between fields, `↑`/`↓` to change the value, or type a number directly. The four fields are Focus, Short Break, Long Break, and Sessions/LB (sessions before a long break). After confirming, a label prompt appears — press `Enter` again to skip it. Press `Esc` to quit.

| Key | Action |
|-----|--------|
| `Space` | Start / pause |
| `n` | Skip to next phase |
| `r` / `gg` | Reset current phase |
| `p` | Switch profile / edit timers |
| `t` | Set task label |
| `[` / `]` | Volume down / up |
| `m` | Mute / unmute |
| `←, h` / `→, l` | Cycle animation themes |
| `↑, k` / `↓, j` | Cycle render modes (Half → Quarter → Braille) |
| `?` | Toggle help overlay |
| `Esc` | Cancel edit / quit |
| `q` / `Ctrl+C` | Quit |

### CLI flags

```sh
tomodoro --help                 # list flags and subcommands
tomodoro --version              # print version
tomodoro --endless              # endless animation mode (also -E)
tomodoro --pause                # pause or resume the running session (IPC)
tomodoro --skip                 # skip to the next phase (IPC)
tomodoro history                # show session history (last 20 rows)
tomodoro history --full         # show complete session history
tomodoro completions bash       # print bash completion script
tomodoro completions zsh        # print zsh completion script
tomodoro completions fish       # print fish completion script
```

To enable tab completion, pipe the output into your shell's completion setup. Examples:

```sh
# bash
tomodoro completions bash > ~/.bash_completion
echo 'source ~/.bash_completion' >> ~/.bashrc

# zsh
mkdir -p ~/.zfunc
tomodoro completions zsh > ~/.zfunc/_tomodoro
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc

# fish
tomodoro completions fish > ~/.config/fish/completions/tomodoro.fish
```

### Endless mode

`tomodoro -E` runs the animated background full-screen with no timer, no session indicators, no progress bar, and no sounds — pure ambient display.

| Key | Action |
|-----|--------|
| `Space` | Pause / resume animation |
| `[` / `]` | Volume down / up |
| `m` | Mute / unmute |
| `←, h` / `→, l` | Cycle animation themes |
| `↑, k` / `↓, j` | Cycle render modes |
| `?` | Show help overlay |
| `q` / `Esc` / `Ctrl+C` | Quit |

## Config

On first launch, `~/.config/tomodoro/config.toml` is created with all options commented out (respects `$XDG_CONFIG_HOME` if set). The generated file is the authoritative reference — every key is documented inline. Open it to explore and uncomment what you need.

Timer profiles are defined as TOML tables and appear in the startup picker:

```toml
[profiles.deep]
focus = 50
short_break = 10
long_break = 30
long_break_interval = 6

[profiles.quick]
focus = 15
short_break = 3
long_break = 10
```

Any field can be omitted — missing values fall back to the scalar defaults above. `long_break_interval` can be set per profile independently of the global value. Custom audio files can be placed in `~/.config/tomodoro/sounds/effects/` (created automatically on first launch).

## Bar integration

Set `waybar_path` in config to a file path — tomodoro writes a JSON status object there on every tick while running, and deletes it on exit:

```toml
waybar_path = "/tmp/tomodoro.json"
```

Output format:

```json
{"text":"F 24:13 2/4","tooltip":"task label","class":"focus"}
```

`class` is `focus`, `short-break`, or `long-break`; `["focus","paused"]` when paused.

Use `--pause` and `--skip` flags to control the running session from outside (waybar clicks, keybinds, scripts, etc.).

### Waybar

```json
"custom/tomodoro": {
    "exec": "cat /tmp/tomodoro.json",
    "return-type": "json",
    "interval": 0,
    "signal": 5,
    "on-click": "/path/to/tomodoro --pause",
    "on-click-right": "/path/to/tomodoro --skip"
}
```

For instant updates instead of polling, set `waybar_signal` to match the waybar `signal` number (1–30, avoid clashes with other modules):

```toml
waybar_signal = 5
```

With `waybar_signal` set, use `"interval": 0` in the waybar module — it only updates on signal.

CSS classes:

```css
#custom-tomodoro { color: #e67e80; }
#custom-tomodoro.short-break { color: #a7c080; }
#custom-tomodoro.long-break  { color: #7fbbb3; }
#custom-tomodoro.paused      { opacity: 0.5; }
```

### Polybar

```ini
[module/tomodoro]
type = custom/script
exec = cat /tmp/tomodoro.json | jq -r '.text'
interval = 1
click-left = tomodoro --pause
click-right = tomodoro --skip
```

### eww

```lisp
(deflisten tomodoro :initial "" "tail -f /tmp/tomodoro.json | jq -r '.text'")
```

## Features

- **Custom durations** — set focus, short break, and long break times on startup or mid-session with `p`/Custom; type values directly or use arrow keys
- **Volume control** — adjust bell and beep volume with `[`/`]`, displayed in the header
- **Session tracker** — dots in the top-right show progress toward a long break; count follows each profile's `long_break_interval` (default 4)
- **Config file** — `~/.config/tomodoro/config.toml` auto-created on first launch; set persistent defaults for themes, durations, volume, and more; invalid or unrecognised values are reset to defaults with an in-app warning; new keys added by updates are merged in automatically without overwriting existing settings
- **Desktop notifications** — optional `notify-send` alerts on phase end; enable with `notifications = true` in config
- **Task labeling** — press `t` mid-session to name the current task; shown in the header; logged with each completed session
- **Session history** — completed focus sessions saved to `~/.local/share/tomodoro/history.json` (respects `$XDG_DATA_HOME`); run `tomodoro history` to see a grouped table by day and task (start time, end time, focus duration, session count) with dashed separators between days and summary stats (avg session length, avg sessions per day, best day); shows last 20 rows by default — pass `--full` for complete history; skipping a focus phase with `n` logs a session if ≥50% of the duration elapsed; quitting mid-focus with `q`/`Esc`/`Ctrl+C` also logs if ≥50% elapsed
- **8 animated themes** — waves, rain, falling leaves, starfield, fireplace, aurora borealis, cherry blossom, sunset; all AI-crafted scenes with detailed foreground elements; set different themes for focus and break phases
- **3 render modes** — half-block, quarter-block, or braille; increasing pixel density per terminal cell
- **Coloured progress bar** — matches the current theme; uses braille dots in braille mode
- **Ambient audio** — looping background track per scene; all 8 themes covered; plays while the timer runs; volume follows `[`/`]`
- **Bell sounds** — single bell when a focus session ends; countdown beeps for the last N seconds of a break (configurable)
- **Phase indicators** — `F` (focus), `B` (short break), `LB` (long break)
- **Endless mode** — `tomodoro -E` runs animations full-screen with no timer, sounds, or UI chrome; `[`/`]` control ambient volume, `m` mutes/unmutes, `?` shows available controls
- **Update check** — checks crates.io on startup and notifies if a newer version is available; dismissible with any key; disable with `update_check = false`
- **Timer profiles** — define named presets in config as `[profiles.name]`; startup shows a picker when profiles exist; selecting a profile auto-labels the session; `default_profile` loads one silently (pairs with `auto_start = true`); press `p` mid-session to switch profiles; switching during a break defers the change until the break ends (shown as `→ name` in the header) — disable with `defer_profile_switch = false`
- **Phase colours** — configure the colour of the phase label (`F`/`B`/`LB`), timer, and session dots per phase via `focus_color`, `short_break_color`, `long_break_color` in config; accepts `#rrggbb`, `#rgb`, `rgb(r,g,b)`, or named colours; import from a TOML or waybar CSS theme file with `color_scheme` and `*_color_key` keys
- **Custom effect sounds** — override the bell and countdown beep with any ogg, mp3, wav, or flac file via `bell_sound` and `beep_sound` in config; place files in `~/.config/tomodoro/sounds/effects/`
- **Bar style** — lock the progress bar to `half`, `quarter`, or `braille` via `bar_style` in config, independent of the animation render mode
- **Daily focus goal** — set `daily_goal_mins` in config to a target number of focus minutes per day; progress shown in the header alongside the session dots; turns green when the goal is met; resets at midnight
- **Fortune popup** — shows a short quote from `fortune` as an overlay at the end of each focus session; dismissible with `q` or `Esc`; silently skipped if `fortune` is not installed
- **What's new popup** — on the first launch after an update, a popup shows the key changes for the new version; dismissible with `q` or `Esc`; scrollable with `↑`/`↓` or `j`/`k`
- **Shell completions** — `tomodoro completions <bash|zsh|fish>` prints a completion script; pipe into your shell's completion setup for tab completion
- **Version management** — install and switch between old releases with `tomodoro install`, `list`, and `--use`

## Version management

Install a specific older version alongside the current one:

```sh
tomodoro install 0.2.2
```

This pulls that version from crates.io and stores it at `~/.local/share/tomodoro/0.2.2/bin/tomodoro`. It does not affect the current binary on your PATH.

List all installed versions:

```sh
tomodoro list
```

Run a specific version:

```sh
tomodoro --use 0.2.2
```

To remove an old version, delete its directory:

```sh
rm -rf ~/.local/share/tomodoro/0.2.2
```

## Requirements

- A terminal with true colour and Unicode support (Ghostty, Kitty, WezTerm, etc.)
- **Linux (Debian/Ubuntu/Mint):** `libasound2-dev` required for audio — install with `sudo apt install libasound2-dev`

## Contributing

Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub theme: usize,
    pub render_mode: String,
    pub focus_theme: Option<usize>,
    pub break_theme: Option<usize>,
    pub focus: u64,
    pub short_break: u64,
    pub long_break: u64,
    pub volume: f32,
    pub long_break_interval: u32,
    pub auto_start: bool,
    pub countdown_beeps: u64,
    pub notifications: bool,
    pub update_check: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: 0,
            render_mode: "half".into(),
            focus_theme: None,
            break_theme: None,
            focus: 25,
            short_break: 5,
            long_break: 15,
            volume: 1.0,
            long_break_interval: 4,
            auto_start: false,
            countdown_beeps: 5,
            notifications: false,
            update_check: true,
        }
    }
}

const DEFAULT_CONFIG: &str = r#"# tomodoro configuration
# All values shown are defaults. Uncomment and edit to customise.

# Starting animation theme (0–7): waves, rain, leaves, stars, fire, aurora, blossom, sunset
# theme = 0

# Per-phase themes — overrides `theme` for each phase independently
# focus_theme = 0
# break_theme = 0

# Render mode: "half", "quarter", or "braille"
# render_mode = "half"

# Default durations in minutes
# focus = 25
# short_break = 5
# long_break = 15

# Sessions before a long break
# long_break_interval = 4

# Starting volume (0.0–1.0)
# volume = 1.0

# Skip the startup screen and begin immediately
# auto_start = false

# Countdown beep seconds at end of each break
# countdown_beeps = 5

# Desktop notifications via notify-send on phase end
# notifications = false

# Check crates.io for a newer version on startup (via cargo search)
# update_check = true
"#;

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(&path, DEFAULT_CONFIG);
            return Self::default();
        }
        let Ok(text) = std::fs::read_to_string(&path) else { return Self::default() };
        toml::from_str(&text).unwrap_or_else(|e| {
            eprintln!("tomodoro: config parse error ({}): {}", path.display(), e);
            Self::default()
        })
    }
}

fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".config/tomodoro/config.toml")
}

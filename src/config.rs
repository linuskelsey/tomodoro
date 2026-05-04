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
    pub bar_style: Option<String>,
    #[serde(skip)]
    pub warnings: Vec<String>,
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
            bar_style: None,
            warnings: Vec::new(),
        }
    }
}

const KNOWN_KEYS: &[&str] = &[
    "theme",
    "render_mode",
    "focus_theme",
    "break_theme",
    "focus",
    "short_break",
    "long_break",
    "volume",
    "long_break_interval",
    "auto_start",
    "countdown_beeps",
    "notifications",
    "update_check",
    "bar_style",
];

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

# Lock the progress bar to a specific style regardless of render mode: "half", "quarter", "braille"
# bar_style = "half"
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

        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("tomodoro: cannot read config {}: {}", path.display(), e);
                std::process::exit(1);
            }
        };

        let table = match text.parse::<toml::Value>() {
            Err(e) => {
                eprintln!("tomodoro: config syntax error in {}:\n  {}", path.display(), e);
                std::process::exit(1);
            }
            Ok(toml::Value::Table(t)) => t,
            Ok(_) => {
                eprintln!("tomodoro: config must be a TOML table ({})", path.display());
                std::process::exit(1);
            }
        };

        let mut config: AppConfig = match toml::from_str(&text) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("tomodoro: config error in {}:\n  {}", path.display(), e);
                std::process::exit(1);
            }
        };

        let mut warnings: Vec<String> = Vec::new();
        let mut dirty_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

        for key in table.keys() {
            if !KNOWN_KEYS.contains(&key.as_str()) {
                warnings.push(format!("'{}' is not a recognised config variable", key));
                dirty_keys.insert(key.clone());
            }
        }

        for (key, msg) in config.validate_and_fix() {
            warnings.push(msg);
            dirty_keys.insert(key);
        }

        let explicit: std::collections::HashSet<String> = table
            .keys()
            .filter(|k| KNOWN_KEYS.contains(&k.as_str()) && !dirty_keys.contains(*k))
            .cloned()
            .collect();

        let migration_needed = KNOWN_KEYS.iter().any(|k| {
            !text.lines().any(|line| {
                let s = line.trim().trim_start_matches('#').trim();
                s.starts_with(&format!("{} =", k)) || s.starts_with(&format!("{}=", k))
            })
        });

        if !dirty_keys.is_empty() || migration_needed {
            let new_content = build_config_content(&config, &explicit);
            let _ = std::fs::write(&path, &new_content);
        }

        config.warnings = warnings;
        config
    }

    fn validate_and_fix(&mut self) -> Vec<(String, String)> {
        let mut fixed: Vec<(String, String)> = Vec::new();
        const VALID_MODES: [&str; 3] = ["half", "quarter", "braille"];

        if self.volume < 0.0 || self.volume > 1.0 {
            fixed.push(("volume".into(), format!(
                "volume = {} is out of range (0.0–1.0)", self.volume
            )));
            self.volume = 1.0;
        }
        if self.theme > 7 {
            fixed.push(("theme".into(), format!(
                "theme = {} is out of range (0–7)", self.theme
            )));
            self.theme = 0;
        }
        if let Some(t) = self.focus_theme {
            if t > 7 {
                fixed.push(("focus_theme".into(), format!(
                    "focus_theme = {} is out of range (0–7)", t
                )));
                self.focus_theme = None;
            }
        }
        if let Some(t) = self.break_theme {
            if t > 7 {
                fixed.push(("break_theme".into(), format!(
                    "break_theme = {} is out of range (0–7)", t
                )));
                self.break_theme = None;
            }
        }
        if !VALID_MODES.contains(&self.render_mode.as_str()) {
            fixed.push(("render_mode".into(), format!(
                "render_mode = '{}' not recognised", self.render_mode
            )));
            self.render_mode = "half".into();
        }
        if let Some(ref s) = self.bar_style.clone() {
            if !VALID_MODES.contains(&s.as_str()) {
                fixed.push(("bar_style".into(), format!(
                    "bar_style = '{}' not recognised", s
                )));
                self.bar_style = None;
            }
        }
        if self.focus == 0 {
            fixed.push(("focus".into(), "focus = 0 is invalid".into()));
            self.focus = 25;
        }
        if self.short_break == 0 {
            fixed.push(("short_break".into(), "short_break = 0 is invalid".into()));
            self.short_break = 5;
        }
        if self.long_break == 0 {
            fixed.push(("long_break".into(), "long_break = 0 is invalid".into()));
            self.long_break = 15;
        }
        if self.long_break_interval == 0 {
            fixed.push(("long_break_interval".into(), "long_break_interval = 0 is invalid".into()));
            self.long_break_interval = 4;
        }

        fixed
    }
}


fn build_config_content(config: &AppConfig, explicit: &std::collections::HashSet<String>) -> String {
    let d = AppConfig::default();
    let mut out = DEFAULT_CONFIG.to_string();

    fn set(content: &mut String, commented: &str, active: &str) {
        *content = content.replace(commented, active);
    }

    if config.theme != d.theme || explicit.contains("theme") {
        set(&mut out, "# theme = 0", &format!("theme = {}", config.theme));
    }
    if config.render_mode != d.render_mode || explicit.contains("render_mode") {
        set(
            &mut out,
            "# render_mode = \"half\"",
            &format!("render_mode = \"{}\"", config.render_mode),
        );
    }
    if config.focus_theme.is_some() || explicit.contains("focus_theme") {
        let t = config.focus_theme.unwrap_or(0);
        set(&mut out, "# focus_theme = 0", &format!("focus_theme = {}", t));
    }
    if config.break_theme.is_some() || explicit.contains("break_theme") {
        let t = config.break_theme.unwrap_or(0);
        set(&mut out, "# break_theme = 0", &format!("break_theme = {}", t));
    }
    if config.focus != d.focus || explicit.contains("focus") {
        set(&mut out, "# focus = 25", &format!("focus = {}", config.focus));
    }
    if config.short_break != d.short_break || explicit.contains("short_break") {
        set(&mut out, "# short_break = 5", &format!("short_break = {}", config.short_break));
    }
    if config.long_break != d.long_break || explicit.contains("long_break") {
        set(&mut out, "# long_break = 15", &format!("long_break = {}", config.long_break));
    }
    if config.long_break_interval != d.long_break_interval || explicit.contains("long_break_interval") {
        set(&mut out, "# long_break_interval = 4", &format!("long_break_interval = {}", config.long_break_interval));
    }
    if (config.volume - d.volume).abs() > 1e-4 || explicit.contains("volume") {
        set(&mut out, "# volume = 1.0", &format!("volume = {}", fmt_float(config.volume)));
    }
    if config.auto_start != d.auto_start || explicit.contains("auto_start") {
        set(&mut out, "# auto_start = false", &format!("auto_start = {}", config.auto_start));
    }
    if config.countdown_beeps != d.countdown_beeps || explicit.contains("countdown_beeps") {
        set(&mut out, "# countdown_beeps = 5", &format!("countdown_beeps = {}", config.countdown_beeps));
    }
    if config.notifications != d.notifications || explicit.contains("notifications") {
        set(&mut out, "# notifications = false", &format!("notifications = {}", config.notifications));
    }
    if config.update_check != d.update_check || explicit.contains("update_check") {
        set(&mut out, "# update_check = true", &format!("update_check = {}", config.update_check));
    }
    if config.bar_style.is_some() || explicit.contains("bar_style") {
        let s = config.bar_style.as_deref().unwrap_or("half");
        set(&mut out, "# bar_style = \"half\"", &format!("bar_style = \"{}\"", s));
    }

    out
}

fn fmt_float(v: f32) -> String {
    let s = format!("{}", v);
    if s.contains('.') {
        s
    } else {
        format!("{}.0", s)
    }
}

fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".config/tomodoro/config.toml")
}

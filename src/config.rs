use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct ProfileConfig {
    pub focus: Option<u64>,
    pub short_break: Option<u64>,
    pub long_break: Option<u64>,
    pub long_break_interval: Option<u32>,
}

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
    pub default_profile: Option<String>,
    pub profiles: std::collections::HashMap<String, ProfileConfig>,
    pub bell_sound: Option<String>,
    pub beep_sound: Option<String>,
    pub defer_profile_switch: bool,
    pub daily_goal_mins: u64,
    pub focus_color: Option<String>,
    pub short_break_color: Option<String>,
    pub long_break_color: Option<String>,
    pub color_scheme: Option<String>,
    pub focus_color_key: Option<String>,
    pub short_break_color_key: Option<String>,
    pub long_break_color_key: Option<String>,
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
            default_profile: None,
            profiles: std::collections::HashMap::new(),
            bell_sound: None,
            beep_sound: None,
            defer_profile_switch: true,
            daily_goal_mins: 0,
            focus_color: None,
            short_break_color: None,
            long_break_color: None,
            color_scheme: None,
            focus_color_key: None,
            short_break_color_key: None,
            long_break_color_key: None,
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
    "default_profile",
    "profiles",
    "bell_sound",
    "beep_sound",
    "defer_profile_switch",
    "daily_goal_mins",
    "focus_color",
    "short_break_color",
    "long_break_color",
    "color_scheme",
    "focus_color_key",
    "short_break_color_key",
    "long_break_color_key",
];

const DEFAULT_CONFIG: &str = r##"# tomodoro configuration
# All values shown are defaults. Uncomment and edit to customise.

# Starting animation theme (0–7): waves, rain, leaves, stars, fire, aurora, blossom, sunset
# theme = 0

# Per-phase themes — overrides `theme` for each phase independently
# focus_theme = 0
# break_theme = 0

# Render mode: "half", "quarter", or "braille"
# render_mode = "half"

# Phase colours — hex (#rrggbb or #rgb), rgb(r,g,b), or a named colour (red, green, cyan, etc.)
# focus_color = "#e67e80"
# short_break_color = "#a7c080"
# long_break_color = "#7fbbb3"

# Import colours from a theme file (.toml or waybar .css)
# color_scheme = "~/.config/omarchy/current_theme.toml"
# focus_color_key = "color1"
# short_break_color_key = "color2"
# long_break_color_key = "color4"

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

# When switching profiles during a break, defer the change until the break ends
# defer_profile_switch = true

# Daily focus goal in minutes — progress shown in header (0 = disabled)
# daily_goal_mins = 0

# Countdown beep seconds at end of each break
# countdown_beeps = 5

# Desktop notifications via notify-send on phase end
# notifications = false

# Check crates.io for a newer version on startup (via cargo search)
# update_check = true

# Lock the progress bar to a specific style regardless of render mode: "half", "quarter", "braille"
# bar_style = "half"

# Profile to load at startup without showing the picker (must match a [profiles.*] name below)
# default_profile = "deep"

# Custom effect sounds — path to an audio file (ogg, mp3, wav, flac)
# Files can be placed in ~/.config/tomodoro/sounds/effects/
# bell_sound = "~/.config/tomodoro/sounds/effects/bell.mp3"
# beep_sound = "~/.config/tomodoro/sounds/effects/beep.mp3"

# Timer profiles — named presets selectable at startup
# Each accepts: focus, short_break, long_break, long_break_interval (minutes/count); omitted values fall back to the defaults above
# [profiles.deep]
# focus = 50
# short_break = 10
# long_break = 30
# long_break_interval = 6
"##;

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
            let _ = std::fs::create_dir_all(dir.join("sounds/effects"));
            let _ = std::fs::create_dir_all(dir.join("sounds/tracks"));
        }
        if !path.exists() {
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

        // Fill missing phase colours from color_scheme file
        if let Some(ref scheme_path) = config.color_scheme.clone() {
            let scheme = load_scheme(scheme_path);
            if config.focus_color.is_none() {
                if let Some(ref key) = config.focus_color_key.clone() {
                    config.focus_color = scheme.get(key.as_str()).cloned();
                }
            }
            if config.short_break_color.is_none() {
                if let Some(ref key) = config.short_break_color_key.clone() {
                    config.short_break_color = scheme.get(key.as_str()).cloned();
                }
            }
            if config.long_break_color.is_none() {
                if let Some(ref key) = config.long_break_color_key.clone() {
                    config.long_break_color = scheme.get(key.as_str()).cloned();
                }
            }
        }

        if let Some(ref dp) = config.default_profile {
            if !config.profiles.contains_key(dp.as_str()) {
                warnings.push(format!("default_profile = '{}' does not match any defined profile", dp));
            }
        }

        for (name, profile) in config.profiles.iter_mut() {
            if profile.focus == Some(0) {
                warnings.push(format!("profiles.{}: focus = 0 is invalid, using default", name));
                profile.focus = None;
            }
            if profile.short_break == Some(0) {
                warnings.push(format!("profiles.{}: short_break = 0 is invalid, using default", name));
                profile.short_break = None;
            }
            if profile.long_break == Some(0) {
                warnings.push(format!("profiles.{}: long_break = 0 is invalid, using default", name));
                profile.long_break = None;
            }
            if profile.long_break_interval == Some(0) {
                warnings.push(format!("profiles.{}: long_break_interval = 0 is invalid, using default", name));
                profile.long_break_interval = None;
            }
        }

        let explicit: std::collections::HashSet<String> = table
            .keys()
            .filter(|k| KNOWN_KEYS.contains(&k.as_str()) && !dirty_keys.contains(*k))
            .cloned()
            .collect();

        let migration_needed = KNOWN_KEYS.iter()
            .filter(|&&k| k != "profiles")
            .any(|k| {
                !text.lines().any(|line| {
                    let s = line.trim().trim_start_matches('#').trim();
                    s.starts_with(&format!("{} =", k)) || s.starts_with(&format!("{}=", k))
                })
            });

        if !dirty_keys.is_empty() || migration_needed {
            let profile_text = extract_profile_sections(&text);
            let mut new_content = build_config_content(&config, &explicit);
            if !profile_text.trim().is_empty() {
                new_content = strip_commented_profile_example(new_content);
                new_content.push('\n');
                new_content.push_str(&profile_text);
            }
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

        if let Some(ref s) = self.focus_color.clone() {
            if parse_color(s).is_none() {
                fixed.push(("focus_color".into(), format!("focus_color = '{}' is not a recognised colour", s)));
                self.focus_color = None;
            }
        }
        if let Some(ref s) = self.short_break_color.clone() {
            if parse_color(s).is_none() {
                fixed.push(("short_break_color".into(), format!("short_break_color = '{}' is not a recognised colour", s)));
                self.short_break_color = None;
            }
        }
        if let Some(ref s) = self.long_break_color.clone() {
            if parse_color(s).is_none() {
                fixed.push(("long_break_color".into(), format!("long_break_color = '{}' is not a recognised colour", s)));
                self.long_break_color = None;
            }
        }

        fixed
    }
}

/// Parse a colour string into (r, g, b). Accepts #rrggbb, #rgb, rgb(r,g,b), or named colours.
pub fn parse_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim().trim_matches('"');
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            return Some((
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            ));
        }
        if hex.len() == 3 {
            return Some((
                u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
            ));
        }
    }
    let s_lower = s.to_lowercase();
    if let Some(inner) = s_lower.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            return Some((
                parts[0].trim().parse::<u8>().ok()?,
                parts[1].trim().parse::<u8>().ok()?,
                parts[2].trim().parse::<u8>().ok()?,
            ));
        }
    }
    match s_lower.as_str() {
        "red"            => Some((255,   0,   0)),
        "green"          => Some((  0, 255,   0)),
        "blue"           => Some((  0,   0, 255)),
        "yellow"         => Some((255, 255,   0)),
        "cyan"           => Some((  0, 255, 255)),
        "magenta"        => Some((255,   0, 255)),
        "white"          => Some((255, 255, 255)),
        "black"          => Some((  0,   0,   0)),
        "orange"         => Some((255, 165,   0)),
        "purple"         => Some((128,   0, 128)),
        "pink"           => Some((255, 192, 203)),
        "teal"           => Some((  0, 128, 128)),
        "coral"          => Some((255, 127,  80)),
        "indigo"         => Some(( 75,   0, 130)),
        "violet"         => Some((238, 130, 238)),
        "gold"           => Some((255, 215,   0)),
        "grey" | "gray"  => Some((128, 128, 128)),
        _                => None,
    }
}

/// Load a flat name→value colour map from a TOML or waybar CSS file.
fn load_scheme(path: &str) -> std::collections::HashMap<String, String> {
    let expanded = if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{}/{}", home, &path[2..])
    } else {
        path.to_string()
    };
    let Ok(text) = std::fs::read_to_string(&expanded) else { return Default::default() };

    if path.ends_with(".css") {
        // Parse waybar-style `@define-color name value;` declarations
        let mut map = std::collections::HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("@define-color") {
                let rest = rest.trim().trim_end_matches(';');
                let mut parts = rest.splitn(2, |c: char| c.is_whitespace());
                if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                    map.insert(name.trim().to_string(), value.trim().to_string());
                }
            }
        }
        map
    } else {
        // TOML: flat key = "value" string pairs
        match text.parse::<toml::Value>() {
            Ok(toml::Value::Table(table)) => table
                .into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect(),
            _ => Default::default(),
        }
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
    if let Some(ref s) = config.default_profile {
        set(&mut out, "# default_profile = \"deep\"", &format!("default_profile = \"{}\"", s));
    }
    if let Some(ref s) = config.bell_sound {
        set(&mut out, "# bell_sound = \"~/.config/tomodoro/sounds/effects/bell.mp3\"", &format!("bell_sound = \"{}\"", s));
    }
    if let Some(ref s) = config.beep_sound {
        set(&mut out, "# beep_sound = \"~/.config/tomodoro/sounds/effects/beep.mp3\"", &format!("beep_sound = \"{}\"", s));
    }
    if config.defer_profile_switch != d.defer_profile_switch || explicit.contains("defer_profile_switch") {
        set(&mut out, "# defer_profile_switch = true", &format!("defer_profile_switch = {}", config.defer_profile_switch));
    }
    if config.daily_goal_mins != d.daily_goal_mins || explicit.contains("daily_goal_mins") {
        set(&mut out, "# daily_goal_mins = 0", &format!("daily_goal_mins = {}", config.daily_goal_mins));
    }
    if let Some(ref s) = config.focus_color {
        set(&mut out, "# focus_color = \"#e67e80\"", &format!("focus_color = \"{}\"", s));
    }
    if let Some(ref s) = config.short_break_color {
        set(&mut out, "# short_break_color = \"#a7c080\"", &format!("short_break_color = \"{}\"", s));
    }
    if let Some(ref s) = config.long_break_color {
        set(&mut out, "# long_break_color = \"#7fbbb3\"", &format!("long_break_color = \"{}\"", s));
    }
    if let Some(ref s) = config.color_scheme {
        set(&mut out, "# color_scheme = \"~/.config/omarchy/current_theme.toml\"", &format!("color_scheme = \"{}\"", s));
    }
    if let Some(ref s) = config.focus_color_key {
        set(&mut out, "# focus_color_key = \"color1\"", &format!("focus_color_key = \"{}\"", s));
    }
    if let Some(ref s) = config.short_break_color_key {
        set(&mut out, "# short_break_color_key = \"color2\"", &format!("short_break_color_key = \"{}\"", s));
    }
    if let Some(ref s) = config.long_break_color_key {
        set(&mut out, "# long_break_color_key = \"color4\"", &format!("long_break_color_key = \"{}\"", s));
    }

    out
}

fn strip_commented_profile_example(content: String) -> String {
    let mut result: Vec<&str> = Vec::new();
    let mut in_example = false;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("# [profiles.") {
            in_example = true;
            continue;
        }
        if in_example {
            if t.starts_with('#') { continue; }
            in_example = false;
        }
        result.push(line);
    }
    while result.last().map_or(false, |l: &&str| l.trim().is_empty()) {
        result.pop();
    }
    result.join("\n") + "\n"
}

fn extract_profile_sections(text: &str) -> String {
    let mut result = String::new();
    let mut in_profile = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[profiles.") {
            in_profile = true;
            result.push_str(line);
            result.push('\n');
        } else if trimmed.starts_with('[') {
            in_profile = false;
        } else if in_profile {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
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
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            std::path::PathBuf::from(home).join(".config")
        });
    base.join("tomodoro/config.toml")
}

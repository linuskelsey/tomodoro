use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub timestamp: String,
    pub duration_mins: u64,
    pub label: Option<String>,
}

pub fn log_session(duration_mins: u64, label: Option<&str>) {
    let path = history_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut sessions: Vec<Session> = load_sessions();
    sessions.push(Session {
        timestamp: timestamp_now(),
        duration_mins,
        label: label.map(|s| s.to_string()),
    });
    if let Ok(json) = serde_json::to_string_pretty(&sessions) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn print_history() {
    let sessions = load_sessions();
    if sessions.is_empty() {
        println!("No sessions recorded.");
        return;
    }
    let total_mins: u64 = sessions.iter().map(|s| s.duration_mins).sum();
    let hours = total_mins / 60;
    let mins = total_mins % 60;

    println!("Sessions:   {}", sessions.len());
    if hours > 0 {
        println!("Total time: {}h {}m", hours, mins);
    } else {
        println!("Total time: {}m", mins);
    }

    let mut tasks: Vec<(String, usize)> = Vec::new();
    for s in &sessions {
        if let Some(label) = &s.label {
            if let Some(entry) = tasks.iter_mut().find(|(l, _)| l == label) {
                entry.1 += 1;
            } else {
                tasks.push((label.clone(), 1));
            }
        }
    }

    if !tasks.is_empty() {
        println!("\nTasks:");
        for (label, count) in &tasks {
            println!("  {} ({} session{})", label, count, if *count == 1 { "" } else { "s" });
        }
    }
}

fn load_sessions() -> Vec<Session> {
    let path = history_path();
    let Ok(text) = std::fs::read_to_string(&path) else { return vec![] };
    serde_json::from_str(&text).unwrap_or_default()
}

fn history_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".local/share/tomodoro/history.json")
}

fn timestamp_now() -> String {
    // ISO 8601 via /bin/date to avoid a chrono dep
    std::process::Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%S")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

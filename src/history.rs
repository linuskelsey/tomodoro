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

pub fn print_history(full: bool) {
    let sessions = load_sessions();
    if sessions.is_empty() {
        println!("No sessions recorded.");
        return;
    }

    let total_mins: u64 = sessions.iter().map(|s| s.duration_mins).sum();
    let avg_session = total_mins / sessions.len() as u64;

    let unique_days: std::collections::HashSet<&str> = sessions.iter()
        .filter_map(|s| s.timestamp.get(..10))
        .collect();
    let avg_per_day = sessions.len() as f64 / unique_days.len() as f64;

    let mut day_mins: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
    for s in &sessions {
        if let Some(day) = s.timestamp.get(..10) {
            *day_mins.entry(day).or_insert(0) += s.duration_mins;
        }
    }
    let best_day = day_mins.iter().max_by_key(|(_, m)| *m);

    println!("Sessions:    {}", sessions.len());
    println!("Avg session: {}", fmt_duration(avg_session));
    println!("Avg day:     {:.1} sessions", avg_per_day);
    if let Some((day, mins)) = best_day {
        println!("Best day:    {} ({})", fmt_date(day), fmt_duration(*mins));
    }
    println!();

    struct Row {
        day: String,
        task: String,
        first_end: String,  // timestamp of first session (recorded at end)
        first_dur: u64,     // duration of first session (to compute start)
        last_end: String,   // timestamp of last session (recorded at end)
        total_dur: u64,
        count: usize,
    }
    let mut rows: Vec<Row> = Vec::new();
    for s in &sessions {
        let day  = s.timestamp.get(..10).unwrap_or("").to_string();
        let time = s.timestamp.get(11..16).unwrap_or("??:??").to_string();
        let task = s.label.clone().unwrap_or_default();
        if let Some(r) = rows.iter_mut().find(|r| r.day == day && r.task == task) {
            r.last_end = time;
            r.total_dur += s.duration_mins;
            r.count += 1;
        } else {
            rows.push(Row { day, task, first_end: time.clone(), first_dur: s.duration_mins, last_end: time, total_dur: s.duration_mins, count: 1 });
        }
    }

    let limit = if full { rows.len() } else { 20 };
    let visible: Vec<&Row> = rows.iter().rev().take(limit).collect();

    let task_w = visible.iter().map(|r| r.task.len().max(4)).max().unwrap_or(4);
    let dur_w = 6usize;
    let sep = "─".repeat(11 + 2 + task_w + 2 + 5 + 2 + 5 + 2 + dur_w + 2 + 1);
    println!("{:<11}  {:<task_w$}  Start  End    {:<dur_w$}  #", "Day", "Task", "Focus", task_w = task_w, dur_w = dur_w);
    println!("{}", sep);
    let mut prev_day: Option<&str> = None;
    for r in &visible {
        if prev_day.map_or(false, |d| d != r.day.as_str()) {
            println!("{}", "╌".repeat(11 + 2 + task_w + 2 + 5 + 2 + 5 + 2 + dur_w + 2 + 1));
        }
        prev_day = Some(&r.day);
        let start = sub_mins_from_time(&r.first_end, r.first_dur);
        let task  = if r.task.is_empty() { "—".to_string() } else { r.task.clone() };
        println!("{:<11}  {:<task_w$}  {}  {}  {:<dur_w$}  {}", fmt_date(&r.day), task, start, r.last_end, fmt_duration(r.total_dur), r.count, task_w = task_w, dur_w = dur_w);
    }
    if !full && rows.len() > 20 {
        println!("\n  {} older rows hidden — run `tomodoro history --full` to see all", rows.len() - 20);
    }
}

fn fmt_date(date: &str) -> String {
    const MONTHS: [&str; 12] = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let mut p = date.splitn(3, '-');
    let (y, mo, d) = match (p.next(), p.next(), p.next()) {
        (Some(y), Some(mo), Some(d)) => (y, mo, d),
        _ => return date.to_string(),
    };
    let month = mo.parse::<usize>().ok()
        .and_then(|n| MONTHS.get(n.saturating_sub(1)))
        .unwrap_or(&"???");
    format!("{} {} {}", d, month, y)
}

fn sub_mins_from_time(time: &str, mins: u64) -> String {
    let h: u64 = time.get(..2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let m: u64 = time.get(3..5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let start = (h * 60 + m + 24 * 60).saturating_sub(mins) % (24 * 60);
    format!("{:02}:{:02}", start / 60, start % 60)
}

fn fmt_duration(mins: u64) -> String {
    let h = mins / 60;
    let m = mins % 60;
    match (h, m) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}

fn load_sessions() -> Vec<Session> {
    let path = history_path();
    let Ok(text) = std::fs::read_to_string(&path) else { return vec![] };
    serde_json::from_str(&text).unwrap_or_default()
}

fn history_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            std::path::PathBuf::from(home).join(".local/share")
        });
    base.join("tomodoro/history.json")
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

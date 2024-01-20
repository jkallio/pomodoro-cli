use std::path::PathBuf;

/// Return the path to the timer information file. This is the cache directory on Linux and
/// LocalAppData on Windows. In case the cache directory is not available, the current
/// directory is used.
pub fn get_timer_info_file() -> PathBuf {
    let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("pomodoro-cli-info.json");
    path
}

/// Return the path to the custom audio file for the alarm. This is the config directory on Linux and RoamingAppData on Windows.
/// In case the audio file is not found, `None` is returned.
pub fn get_custom_alarm_file() -> Option<PathBuf> {
    if let Some(mut path) = dirs::config_dir() {
        path.push("pomodoro-cli");
        path.push("alarm.mp3");
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Return the path to the custom icon file for the notification. This is the config directory on Linux and RoamingAppData on Windows.
/// In case the icon file is not found, `None` is returned.
/// The icon file must be a PNG file.
pub fn get_custom_icon_file() -> Option<PathBuf> {
    if let Some(mut path) = dirs::config_dir() {
        path.push("pomodoro-cli");
        path.push("icon.png");
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// The duration can be passed either as a number (as minutes) or as string in the format of "1h 30m 10s"
pub fn parse_duration(duration: Option<String>) -> Option<i64> {
    if let Some(duration) = duration {
        if let Ok(duration) = duration.parse::<i64>() {
            return Some(duration * 60);
        }

        let mut duration = duration.to_lowercase();
        duration.retain(|c| !c.is_whitespace());
        duration = duration.replace("hour", "h");
        duration = duration.replace("minute", "m");
        duration = duration.replace("min", "m");
        duration = duration.replace("second", "s");
        duration = duration.replace("sec", "s");

        let mut hours = 0;
        let mut minutes = 0;
        let mut seconds = 0;
        if duration.contains("h") {
            duration.split("h");
            let parts = duration.split("h").collect::<Vec<&str>>();
            hours = parts[0].parse().unwrap_or_default();
            duration = parts[1].to_string();
        }
        if duration.contains("m") {
            duration.split("m");
            let parts = duration.split("m").collect::<Vec<&str>>();
            minutes = parts[0].parse().unwrap_or_default();
            duration = parts[1].to_string();
        }
        if duration.contains("s") {
            duration.split("s");
            let parts = duration.split("s").collect::<Vec<&str>>();
            seconds = parts[0].parse().unwrap_or_default();
        }
        return Some(hours * 60 * 60 + minutes * 60 + seconds);
    }
    None
}

/// Return the seconds in human-readable format (e.g. 1h 30m 10s)
pub fn convert_to_text_format(seconds: i64) -> String {
    let mut seconds = seconds;
    let hours = seconds / 3600;
    seconds -= hours * 3600;
    let minutes = (seconds % 3600) / 60;
    seconds -= minutes * 60;

    let mut time = String::new();
    if hours > 0 {
        time.push_str(&format!("{}h", hours));
    }
    if minutes > 0 {
        if !time.is_empty() {
            time.push_str(" ");
        }
        time.push_str(&format!("{}m", minutes));
    }
    if seconds > 0 {
        if !time.is_empty() {
            time.push_str(" ");
        }
        time.push_str(&format!("{}s", seconds));
    }
    if time.is_empty() {
        time.push_str("0s");
    }
    return time;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration(Some("1h 30m 10s".to_string())), Some(5410));
        assert_eq!(
            parse_duration(Some("1H 30Min 10SeC".to_string())),
            Some(5410)
        );
        assert_eq!(parse_duration(Some("2h15m1s".to_string())), Some(8101));
        assert_eq!(parse_duration(Some("1h 30m".to_string())), Some(5400));
        assert_eq!(parse_duration(Some("1hour".to_string())), Some(3600));
        assert_eq!(parse_duration(Some("30m 10s".to_string())), Some(1810));
        assert_eq!(parse_duration(Some("30m".to_string())), Some(1800));
        assert_eq!(parse_duration(Some("10s".to_string())), Some(10));
        assert_eq!(parse_duration(Some("100".to_string())), Some(100 * 60));
        assert_eq!(parse_duration(Some("Invalid string".to_string())), Some(0));
    }

    #[test]
    fn test_get_human_readable_time() {
        assert_eq!(convert_to_text_format(5410), "1h 30m 10s");
        assert_eq!(convert_to_text_format(60), "1m");
        assert_eq!(convert_to_text_format(10), "10s");
        assert_eq!(convert_to_text_format(0), "0s");
    }
}

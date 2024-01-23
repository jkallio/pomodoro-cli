use crate::error::*;
use std::path::PathBuf;

/// Create a valid file name from a String
pub fn create_filename_from_string(name: &str) -> String {
    let mut filename = name.to_lowercase();
    filename.retain(|c| c.is_ascii_alphanumeric() || c == '_');
    return filename;
}

/// Create a new folder if it doesn't exist
pub fn ensure_folder_exists(path: &PathBuf) -> AppResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Return the path to a file inside a cache directory. This is the cache directory on Linux and LocalAppData on Windows.
pub fn create_cache_file_path(filename: &str) -> AppResult<PathBuf> {
    if let Some(mut path) = dirs::cache_dir() {
        path.push("pomodoro-cli");
        ensure_folder_exists(&path)?;
        path.push(filename);
        return Ok(path);
    }
    Err(AppError::new("Failed to locate AppData / Cache directory"))
}

/// Return the path to a file inside a config directory. This is the config directory on Linux and RoamingAppData on Windows.
pub fn create_config_file_path(filename: &str) -> AppResult<PathBuf> {
    if let Some(mut path) = dirs::config_dir() {
        path.push("pomodoro-cli");
        ensure_folder_exists(&path)?;
        path.push(filename);
        return Ok(path);
    }
    Err(AppError::new("Failed to locate AppData / Config directory"))
}

/// The duration can be passed either as a number (as minutes) or as string in the format of "1h 30m 10s"
pub fn parse_duration(duration: &Option<String>) -> AppResult<i64> {
    if let Some(duration) = duration {
        if let Ok(duration) = duration.parse::<i64>() {
            return Ok(duration * 60);
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
        let duration = hours * 60 * 60 + minutes * 60 + seconds;
        if duration > 0 {
            return Ok(duration);
        }
    }
    Err(AppError::new("Failed to parse duration"))
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
        assert_eq!(
            parse_duration(&Some("1h 30m 10s".to_string())).unwrap(),
            5410
        );
        assert_eq!(
            parse_duration(&Some("1H 30Min 10SeC".to_string())).unwrap(),
            5410
        );
        assert_eq!(parse_duration(&Some("2h15m1s".to_string())).unwrap(), 8101);
        assert_eq!(parse_duration(&Some("1h 30m".to_string())).unwrap(), 5400);
        assert_eq!(parse_duration(&Some("1hour".to_string())).unwrap(), 3600);
        assert_eq!(parse_duration(&Some("30m 10s".to_string())).unwrap(), 1810);
        assert_eq!(parse_duration(&Some("30m".to_string())).unwrap(), 1800);
        assert_eq!(parse_duration(&Some("10s".to_string())).unwrap(), 10);
        assert_eq!(parse_duration(&Some("100".to_string())).unwrap(), 100 * 60);
        assert!(parse_duration(&Some("Invalid string".to_string())).is_err());
    }

    #[test]
    fn test_get_human_readable_time() {
        assert_eq!(convert_to_text_format(5410), "1h 30m 10s");
        assert_eq!(convert_to_text_format(60), "1m");
        assert_eq!(convert_to_text_format(10), "10s");
        assert_eq!(convert_to_text_format(0), "0s");
    }
}

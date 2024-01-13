use crate::args::Args;
use crate::utils::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

const DEFAULT_DURATION: i64 = 25 * 60;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TimerState {
    Running,
    Stopped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerInfo {
    pub state: TimerState,
    pub start_time: i64,
    pub duration: i64,
    pub silent: bool,
}
impl Default for TimerInfo {
    fn default() -> Self {
        let start_time = chrono::Utc::now().timestamp();
        Self {
            state: TimerState::Stopped,
            start_time,
            duration: DEFAULT_DURATION,
            silent: false,
        }
    }
}
impl TimerInfo {
    pub fn from_args(args: &Args) -> Self {
        let mut timer_info = Self::default();
        if let Some(d) = &args.duration {
            timer_info.duration = parse_duration(&d);
        }
        timer_info.start_time = chrono::Utc::now().timestamp();
        timer_info.silent = args.silent;
        timer_info
    }

    pub fn from_file() -> Self {
        let path = get_state_file();
        if !path.exists() {
            return Self::default();
        }
        let mut file = std::fs::File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str(&contents).unwrap()
    }

    pub fn is_finished(&self) -> bool {
        self.get_time_left() == 0
    }

    pub fn get_time_left(&self) -> i64 {
        let now = chrono::Utc::now().timestamp();
        let time_left = self.start_time + self.duration - now;
        return i64::max(0, time_left);
    }

    pub fn get_time_elapsed(&self) -> i64 {
        let now = chrono::Utc::now().timestamp();
        let time_elapsed = now - self.start_time;
        return i64::max(0, time_elapsed);
    }

    pub fn write_to_file(&self) {
        let path = get_state_file();
        let mut file = File::create(path).unwrap();
        let json = serde_json::to_string_pretty(&self).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    #[allow(dead_code)]
    pub fn remove_info_file() {
        let path = get_state_file();
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
    }

    #[allow(dead_code)]
    pub fn info_file_exists() -> bool {
        let path = get_state_file();
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_io() {
        TimerInfo::remove_info_file();
        assert!(!TimerInfo::info_file_exists());
        let timer_info = TimerInfo::default();
        timer_info.write_to_file();
        assert!(TimerInfo::info_file_exists());
        TimerInfo::remove_info_file();
    }
}

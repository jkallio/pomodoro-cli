use crate::args::Args;
use crate::utils::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

const DEFAULT_DURATION: u64 = 25 * 60;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TimerState {
    Running,
    Stopped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerInfo {
    pub state: TimerState,
    pub start_time: u64,
    pub pause_time: u64,
    pub duration: u64,
    pub silent: bool,
}
impl Default for TimerInfo {
    fn default() -> Self {
        let start_time = chrono::Utc::now().timestamp() as u64;
        Self {
            state: TimerState::Stopped,
            start_time,
            pause_time: start_time,
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
        timer_info.start_time = chrono::Utc::now().timestamp() as u64;
        timer_info.pause_time = timer_info.start_time;
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

    pub fn get_time_left(&self) -> u64 {
        let now = chrono::Utc::now().timestamp() as u64;
        let time_left = self.start_time + self.duration - now;
        if time_left > 0 {
            return time_left;
        }
        0
    }

    pub fn get_time_elapsed(&self) -> u64 {
        let now = chrono::Utc::now().timestamp() as u64;
        let time_elapsed = now - self.start_time;
        if time_elapsed > 0 {
            return time_elapsed;
        }
        0
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

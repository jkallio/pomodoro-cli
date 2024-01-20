use crate::error::*;
use crate::utils::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

/// The default duration of the timer in seconds
pub const DEFAULT_TIMER_DURATION: i64 = 25 * 60;

/// Defines the state of the timer
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TimerState {
    Running,
    Paused,
    Finished,
}

/// Defines the timer info data structure (which is stored as JSON in system cache directory)
#[derive(Debug, Serialize, Deserialize)]
pub struct TimerInfo {
    pub state: TimerState,
    pub start_time: i64,
    pub pause_time: i64,
    pub duration: i64,
    pub silent: bool,
    pub notify: bool,
    pub wait: bool,
}

#[derive(Serialize)]
pub struct WaybarTimerInfo {
    pub text: String,
    pub tooltip: String,
    pub class: String,
    pub percentage: f64,
}

/// Implement default for TimerInfo
impl Default for TimerInfo {
    fn default() -> Self {
        let start_time = chrono::Utc::now().timestamp();
        Self {
            state: TimerState::Paused,
            start_time,
            pause_time: start_time,
            duration: DEFAULT_TIMER_DURATION,
            silent: false,
            notify: false,
            wait: false,
        }
    }
}

/// Implement convinience methods for TimerInfo
impl TimerInfo {
    /// Initialize the TimerInfo from the stored JSON file. Defaults to default values if the file does not exist.
    pub fn from_file_or_default() -> AppResult<Self> {
        let path = get_timer_info_file();
        if !path.exists() {
            return Ok(Self::default());
        }

        let mut contents = String::new();
        let mut file = std::fs::File::open(path)?;

        file.read_to_string(&mut contents)?;
        if let Ok(timer_info) = serde_json::from_str(&contents) {
            return Ok(timer_info);
        }
        Ok(Self::default())
    }

    /// Return true if the timer is in `Running` state
    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    /// Return true if the timer is in `Paused` state
    pub fn is_paused(&self) -> bool {
        self.state == TimerState::Paused
    }

    pub fn is_time_run_out(&self) -> bool {
        self.get_time_left() <= 0
    }

    /// Returns the time left in the timer in seconds.
    pub fn get_time_left(&self) -> i64 {
        self.duration - self.get_time_elapsed()
    }

    /// Returns the info in Waybar JSON format.
    pub fn get_json_info(&self) -> AppResult<String> {
        let time_left = self.get_time_left();
        let percentage = (time_left as f64 / self.duration as f64) * 100.0;
        let text = get_human_readable_time(time_left);
        let tooltip = match self.state {
            TimerState::Running => format!(
                "Running\nLeft: {}\nElapsed: {} ",
                get_human_readable_time(self.get_time_left()),
                get_human_readable_time(self.get_time_elapsed())
            ),
            TimerState::Paused => format!(
                "Paused\nLeft: {}\nElapsed: {} ",
                get_human_readable_time(self.get_time_left()),
                get_human_readable_time(self.get_time_elapsed())
            ),
            TimerState::Finished => "Finished".to_string(),
        }
        .to_string();
        let class = match self.state {
            TimerState::Running => "running",
            TimerState::Paused => "paused",
            TimerState::Finished => "finished",
        };
        let waybar_info = WaybarTimerInfo {
            text,
            tooltip,
            class: class.to_string(),
            percentage,
        };
        return Ok(serde_json::to_string(&waybar_info)?);
    }

    /// Returns the time elapsed since start in seconds.
    pub fn get_time_elapsed(&self) -> i64 {
        match self.state {
            TimerState::Finished => return self.duration,
            TimerState::Paused => return self.pause_time - self.start_time,
            TimerState::Running => {
                let now = chrono::Utc::now().timestamp();
                let time_elapsed = now - self.start_time;
                return i64::max(0, time_elapsed);
            }
        }
    }

    /// Write the TimerInfo to the JSON file.
    pub fn write_to_file(&self) -> AppResult<()> {
        let path = get_timer_info_file();
        let mut file = File::create(path)?;
        let json = serde_json::to_string_pretty(&self)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Remove the JSON file from the system cache directory.
    #[allow(dead_code)]
    pub fn remove_info_file() -> AppResult<()> {
        let path = get_timer_info_file();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Returns true if the JSON file exists in the system cache directory.
    #[allow(dead_code)]
    pub fn info_file_exists() -> bool {
        let path = get_timer_info_file();
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_io() -> AppResult<()> {
        TimerInfo::remove_info_file()?;
        assert!(!TimerInfo::info_file_exists());
        let timer_info = TimerInfo::default();
        timer_info.write_to_file()?;
        assert!(TimerInfo::info_file_exists());
        TimerInfo::remove_info_file()?;
        Ok(())
    }

    #[test]
    fn test_time_left() {
        let now = chrono::Utc::now().timestamp();
        let mut timer_info = TimerInfo::default();
        timer_info.start_time = now - 10;
        timer_info.duration = 20;
        assert_eq!(timer_info.get_time_left(), 10);
    }

    #[test]
    fn test_time_elapsed() {
        let now = chrono::Utc::now().timestamp();
        let mut timer_info = TimerInfo::default();
        timer_info.start_time = now - 10;
        timer_info.duration = 20;
        assert_eq!(timer_info.get_time_elapsed(), 10);
    }
}

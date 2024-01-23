use crate::error::*;
use crate::utils::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

/// Defines the reusable TimerProfile struct
#[derive(Serialize, Deserialize, Debug)]
pub struct TimerProfile {
    pub name: String,
    pub sequence: Vec<i64>,
    pub messages: Vec<String>,
    pub alert_path: String,
    pub icon_path: String,
    pub silent: bool,
    pub notify: bool,
    pub repeat: u32,
}

impl TimerProfile {
    /// Write the TimerProfile as JSON file into system AppData / Config directory
    pub fn write_to_file(&self) -> AppResult<()> {
        let filename = create_filename_from_string(&self.name);
        let path = create_config_file_path(&filename)?;
        let mut file = File::create(path)?;
        let json = serde_json::to_string_pretty(&self)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

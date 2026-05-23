use crate::error::*;
use crate::utils::get_cycle_file;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CyclePhase {
    pub name: String,
    pub minutes: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CycleDefinition {
    pub phases: Vec<CyclePhase>,
}

const DEFAULT_PHASES: &[(&str, u32)] = &[
    ("Work", 25),
    ("Short Break", 5),
    ("Work", 25),
    ("Short Break", 5),
    ("Work", 25),
    ("Short Break", 5),
    ("Work", 25),
    ("Long Break", 30),
];

impl CycleDefinition {
    pub fn default_cycle() -> Self {
        Self {
            phases: DEFAULT_PHASES
                .iter()
                .map(|(name, minutes)| CyclePhase {
                    name: name.to_string(),
                    minutes: *minutes,
                })
                .collect(),
        }
    }

    pub fn load_or_default() -> Self {
        let path = get_cycle_file();
        if path.exists()
            && let Ok(contents) = std::fs::read_to_string(&path)
            && let Ok(cycle) = serde_json::from_str(&contents)
        {
            return cycle;
        }
        Self::default_cycle()
    }

    pub fn save(&self) -> AppResult<()> {
        let path = get_cycle_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        let json = serde_json::to_string_pretty(self)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    pub fn phase(&self, index: usize) -> &CyclePhase {
        &self.phases[index]
    }

    pub fn len(&self) -> usize {
        self.phases.len()
    }

    pub fn parse_phases(args: &[String]) -> AppResult<Self> {
        let mut phases = Vec::new();
        for arg in args {
            let (name, minutes_str) = arg.rsplit_once(':').ok_or_else(|| {
                AppError::new(&format!(
                    "Invalid phase format '{}'. Use 'Name:minutes' (e.g. 'Work:25').",
                    arg
                ))
            })?;
            let minutes: u32 = minutes_str.parse().map_err(|_| {
                AppError::new(&format!(
                    "Invalid duration '{}' in '{}'. Must be a positive integer.",
                    minutes_str, arg
                ))
            })?;
            if minutes == 0 {
                return Err(AppError::new(&format!(
                    "Duration must be greater than 0 in '{}'.",
                    arg
                )));
            }
            phases.push(CyclePhase {
                name: name.to_string(),
                minutes,
            });
        }
        Ok(Self { phases })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_default_cycle_has_eight_phases() {
        let cycle = CycleDefinition::default_cycle();
        assert_eq!(cycle.len(), 8);
    }

    #[test]
    fn test_parse_phases_valid() {
        let args = vec!["Work:25".to_string(), "Break:5".to_string()];
        let cycle = CycleDefinition::parse_phases(&args).unwrap();
        assert_eq!(cycle.len(), 2);
        assert_eq!(cycle.phase(0).name, "Work");
        assert_eq!(cycle.phase(0).minutes, 25);
        assert_eq!(cycle.phase(1).name, "Break");
        assert_eq!(cycle.phase(1).minutes, 5);
    }

    #[test]
    fn test_parse_phases_name_with_spaces() {
        let args = vec!["Short Break:5".to_string()];
        let cycle = CycleDefinition::parse_phases(&args).unwrap();
        assert_eq!(cycle.phase(0).name, "Short Break");
        assert_eq!(cycle.phase(0).minutes, 5);
    }

    #[test]
    fn test_parse_phases_missing_colon() {
        let args = vec!["Work25".to_string()];
        assert!(CycleDefinition::parse_phases(&args).is_err());
    }

    #[test]
    fn test_parse_phases_invalid_minutes() {
        let args = vec!["Work:abc".to_string()];
        assert!(CycleDefinition::parse_phases(&args).is_err());
    }

    #[test]
    fn test_parse_phases_zero_minutes() {
        let args = vec!["Work:0".to_string()];
        assert!(CycleDefinition::parse_phases(&args).is_err());
    }

    #[test]
    #[serial]
    fn test_save_and_load() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        unsafe {
            std::env::set_var("POMODORO_CLI_TEST_DIR", dir.path());
        }

        let cycle =
            CycleDefinition::parse_phases(&["Work:25".to_string(), "Break:5".to_string()]).unwrap();
        cycle.save().unwrap();

        let loaded = CycleDefinition::load_or_default();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.phase(0).name, "Work");
        assert_eq!(loaded.phase(1).minutes, 5);
    }
}

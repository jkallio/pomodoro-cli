use crate::args::*;
use crate::cycle::CycleDefinition;
use crate::error::*;
use crate::timer_info::DEFAULT_TIMER_DURATION;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use crossterm::cursor::{MoveToColumn, MoveToPreviousLine};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use lock::FailureReason;
use notify_rust::{Notification, Timeout};
use rodio::{Decoder, Player, stream::DeviceSinkBuilder};
use std::thread;
use std::time::Duration;

/// Run the application with the given arguments
pub fn run(args: &Cli) -> AppResult<()> {
    match &args.subcmd {
        SubCommand::SetCycle { phases } => {
            set_cycle(phases)?;
        }
        SubCommand::Start {
            duration,
            add,
            message,
            silent,
            notify,
            wait,
            resume,
            lock_screen,
            cycle,
            repeat,
        } => {
            let parsed_duration = parse_duration(duration.clone());
            let parsed_add = parse_duration(add.clone());

            if duration.is_some() && parsed_duration.is_none() {
                return Err(AppError::new(&format!(
                    "Invalid duration format: '{}'. Use formats like '25m', '1h 30m 10s', or '10:30'",
                    duration.as_ref().unwrap()
                )));
            }

            if add.is_some() && parsed_add.is_none() {
                return Err(AppError::new(&format!(
                    "Invalid time format: '{}'. Use formats like '5m', '1h 30m', or '10:30'",
                    add.as_ref().unwrap()
                )));
            }

            start_timer(StartTimerArgs {
                duration: parsed_duration,
                add: parsed_add,
                message: message.clone().unwrap_or("".to_string()),
                silent: *silent,
                notify: *notify,
                wait: *wait,
                resume: *resume,
                lock_screen: *lock_screen,
                cycle: *cycle,
                repeat: *repeat,
            })?;

            if *wait || *lock_screen {
                wait_for_timer()?;
            }
        }
        SubCommand::Pause => {
            pause_timer()?;
        }
        SubCommand::Stop => {
            stop_timer()?;
        }
        SubCommand::Status {
            format,
            time_format,
            watch,
        } => {
            if *watch {
                watch_status(*format, *time_format)?;
            } else {
                let status = get_status(*format, *time_format)?;
                println!("{}", status);
            }
        }
    }
    Ok(())
}

/// Arguments for starting a timer.
#[derive(Default)]
pub struct StartTimerArgs {
    pub duration: Option<i64>,
    pub add: Option<i64>,
    pub message: String,
    pub silent: bool,
    pub notify: bool,
    pub wait: bool,
    pub resume: bool,
    pub lock_screen: bool,
    pub cycle: bool,
    pub repeat: bool,
}

/// Start the timer. If the timer is already running, the duration is added to the current duration.
pub fn start_timer(args: StartTimerArgs) -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    if let Some(time) = args.add
        && timer_info.is_running()
    {
        // Add more time to the running timer
        timer_info.duration += time;
    } else if timer_info.is_paused() && args.resume {
        // Resume a paused timer
        let now = chrono::Utc::now().timestamp();
        let elapsed = timer_info.pause_time - timer_info.start_time;
        timer_info.duration -= elapsed;
        timer_info.start_time = now;
        timer_info.pause_time = now;
        timer_info.silent = timer_info.silent || args.silent;
        timer_info.notify = timer_info.notify || args.notify;
        timer_info.wait = timer_info.wait || args.wait;
        timer_info.lock_screen = timer_info.lock_screen || args.lock_screen;
        timer_info.state = TimerState::Running;
    } else if args.cycle {
        // Start the first phase of the cycle
        let cycle = CycleDefinition::load_or_default();
        let first = cycle.phase(0);
        let now = chrono::Utc::now().timestamp() + 1;
        timer_info.duration = first.minutes as i64 * 60;
        timer_info.start_time = now;
        timer_info.pause_time = now;
        timer_info.message = args.message;
        timer_info.silent = args.silent;
        timer_info.notify = args.notify;
        timer_info.wait = args.wait;
        timer_info.lock_screen = args.lock_screen;
        timer_info.state = TimerState::Running;
        timer_info.cycle_phase_index = Some(0);
        timer_info.cycle_total = Some(cycle.len());
        timer_info.cycle_phase_name = first.name.clone();
        timer_info.repeat = args.repeat;
    } else {
        // Start a new single timer; when --repeat is given without an explicit duration,
        // reuse the previous timer's duration so the session can continue seamlessly.
        let duration = if args.repeat && args.duration.is_none() && args.add.is_none() {
            timer_info.duration
        } else {
            args.duration
                .unwrap_or(args.add.unwrap_or(DEFAULT_TIMER_DURATION))
        };
        let now = chrono::Utc::now().timestamp() + 1;
        timer_info.duration = duration;
        timer_info.start_time = now;
        timer_info.pause_time = now;
        timer_info.message = args.message;
        timer_info.silent = args.silent;
        timer_info.notify = args.notify;
        timer_info.wait = args.wait;
        timer_info.state = TimerState::Running;
        timer_info.lock_screen = args.lock_screen;
        timer_info.cycle_phase_index = None;
        timer_info.cycle_total = None;
        timer_info.cycle_phase_name = String::new();
        timer_info.repeat = args.repeat;
    }
    timer_info.write_to_file()?;
    Ok(())
}

/// Save a cycle definition from name:minutes pairs, or reset to the built-in default.
pub fn set_cycle(phase_args: &[String]) -> AppResult<()> {
    let cycle = if phase_args.is_empty() {
        CycleDefinition::default_cycle()
    } else {
        CycleDefinition::parse_phases(phase_args)?
    };
    cycle.save()?;
    println!("Cycle saved ({} phases):", cycle.len());
    for (i, phase) in cycle.phases.iter().enumerate() {
        println!("  {:>2}. {:<20} {} min", i + 1, phase.name, phase.minutes);
    }
    Ok(())
}

/// Advance a cycle timer to the next phase. Returns true if a next phase was started,
/// false if the cycle has ended (and the caller should stop the timer).
fn advance_cycle_phase(timer_info: &mut TimerInfo) -> AppResult<bool> {
    let Some(current_index) = timer_info.cycle_phase_index else {
        return Ok(false);
    };
    let total = timer_info.cycle_total.unwrap_or(0);
    let next_index = current_index + 1;

    let cycle = CycleDefinition::load_or_default();

    let phase_index = if next_index < total && next_index < cycle.len() {
        next_index
    } else if timer_info.repeat {
        0
    } else {
        return Ok(false);
    };

    let phase = cycle.phase(phase_index);
    let now = chrono::Utc::now().timestamp() + 1;
    timer_info.duration = phase.minutes as i64 * 60;
    timer_info.start_time = now;
    timer_info.pause_time = now;
    timer_info.cycle_phase_index = Some(phase_index);
    timer_info.cycle_phase_name = phase.name.clone();
    timer_info.state = TimerState::Running;
    timer_info.write_to_file()?;
    Ok(true)
}

/// Pause the timer. If the timer is already paused, the timer is resumed.
pub fn pause_timer() -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    if timer_info.is_paused() {
        start_timer(StartTimerArgs {
            duration: Some(timer_info.duration),
            message: timer_info.message,
            silent: timer_info.silent,
            notify: timer_info.notify,
            wait: timer_info.wait,
            resume: true,
            lock_screen: timer_info.lock_screen,
            ..Default::default()
        })?;
    } else if timer_info.is_running() {
        let now = chrono::Utc::now().timestamp();
        timer_info.pause_time = now;
        timer_info.state = TimerState::Paused;
        timer_info.write_to_file()?;
    }
    Ok(())
}

/// Stop the timer.
pub fn stop_timer() -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    timer_info.state = TimerState::Finished;
    timer_info.write_to_file()?;
    Ok(())
}

/// Lock the screen.
fn lock_screen() -> AppResult<()> {
    println!("Locking screen...");

    lock::lock().map_err(|fail| {
        AppError::new(match fail {
            FailureReason::CannotExecute => "Cannot execute the lock command.",
            FailureReason::LinuxCommandNotFound => {
                "Linux command not found. The following commands are supported\
                    \n- xdg-screensaver\
                    \n- gnome-screensaver\
                    \n- dm-tool"
            }
        })
    })
}

/// Trigger the alarm sound and/or the system notification.
pub fn trigger_alarm(timer_info: &TimerInfo) -> AppResult<()> {
    println!("Time is up!");

    if timer_info.notify {
        let mut path = String::from("dialog-warning");
        if let Some(custom_icon_path) = get_custom_icon_file() {
            path = custom_icon_path.to_str().unwrap_or(&path).to_string();
        }
        Notification::new()
            .summary("Pomodoro Timer")
            .body("Time is up!")
            .icon(&path)
            .appname("pomodoro-cli")
            .timeout(Timeout::from(Duration::from_secs(300)))
            .show()?;
    }

    if !timer_info.silent {
        let mut handle = DeviceSinkBuilder::open_default_sink()?;
        handle.log_on_drop(false);
        let player = Player::connect_new(handle.mixer());
        if let Some(path) = get_custom_alarm_file() {
            let file = std::fs::File::open(path)?;
            let source = Decoder::new(file)?;
            player.append(source);
        } else {
            let mp3 = include_bytes!("../assets/ding.mp3");
            let source = Decoder::new(std::io::Cursor::new(mp3))?;
            player.append(source);
        }
        player.set_volume(1.0);
        player.sleep_until_end();
        player.clear();
    }

    if timer_info.lock_screen {
        lock_screen()?;
    }

    Ok(())
}

/// Return the status of the timer in the given format.
pub fn get_status(
    format: Option<StatusFormat>,
    time_format: Option<TimeFormat>,
) -> AppResult<String> {
    let mut timer_info = TimerInfo::from_file_or_default()?;

    let timed_out = timer_info.is_running() && !timer_info.wait && timer_info.is_time_run_out();
    let is_cycle = timer_info.cycle_phase_index.is_some();

    if timed_out && is_cycle {
        // Trigger alarm for the completed phase, then advance to the next one
        trigger_alarm(&timer_info)?;
        if !advance_cycle_phase(&mut timer_info)? {
            stop_timer()?;
            timer_info.state = TimerState::Finished;
        }
    } else if timed_out {
        if !timer_info.repeat {
            stop_timer()?;
        }
        timer_info.state = TimerState::Finished;
    }

    let status = match format {
        Some(StatusFormat::Json) => timer_info.get_json_info(time_format.unwrap_or_default())?,
        _ => timer_info.get_human_readable(time_format.unwrap_or_default()),
    };

    if timed_out && !is_cycle {
        trigger_alarm(&timer_info)?;
        if timer_info.repeat {
            let now = chrono::Utc::now().timestamp() + 1;
            timer_info.start_time = now;
            timer_info.pause_time = now;
            timer_info.state = TimerState::Running;
            timer_info.write_to_file()?;
        }
    }

    Ok(status)
}

/// Stream status to stdout once per second (Waybar streaming/daemon mode).
pub fn watch_status(
    format: Option<StatusFormat>,
    time_format: Option<TimeFormat>,
) -> AppResult<()> {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    loop {
        let status = get_status(format, time_format)?;
        writeln!(stdout, "{}", status)?;
        stdout.flush()?;
        thread::sleep(Duration::from_secs(1));
    }
}

/// Wait for the timer to finish, displaying a progress bar.
///
/// Blocks until all phases complete (cycle mode) or the single timer expires/is stopped.
pub fn wait_for_timer() -> AppResult<()> {
    let mut stdout = std::io::stdout();

    loop {
        let mut timer_info = TimerInfo::from_file_or_default()?;

        let percentage = (timer_info.get_percentage() / 4.0).clamp(0.0, 25.0) as i64;
        print!("|");
        for _ in 0..percentage {
            print!("#");
        }
        for _ in 0..(25 - percentage) {
            print!("-");
        }
        println!("| {}", timer_info.get_human_readable(TimeFormat::default()));

        thread::sleep(Duration::from_secs(1));

        execute!(
            stdout,
            MoveToPreviousLine(1),
            Clear(ClearType::CurrentLine),
            MoveToColumn(0),
        )?;

        if !timer_info.is_running() {
            stop_timer()?;
            break;
        }

        if timer_info.is_time_run_out() {
            trigger_alarm(&timer_info)?;
            if timer_info.cycle_phase_index.is_some() && advance_cycle_phase(&mut timer_info)? {
                // Phase advanced; loop continues for the next phase
                continue;
            }
            if timer_info.repeat {
                let now = chrono::Utc::now().timestamp() + 1;
                timer_info.start_time = now;
                timer_info.pause_time = now;
                timer_info.state = TimerState::Running;
                timer_info.write_to_file()?;
                continue;
            }
            stop_timer()?;
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let dir = TempDir::new().unwrap();
        unsafe {
            std::env::set_var("POMODORO_CLI_TEST_DIR", dir.path());
        }
        std::thread::sleep(Duration::from_millis(10));
        let _ = TimerInfo::remove_info_file();
        dir
    }

    #[test]
    #[serial]
    fn test_start_timer_new() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Running);
        assert_eq!(timer_info.duration, 60);
        assert_eq!(timer_info.message, "");

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_start_timer_with_message() {
        let _temp = setup_test_env();

        let message = "Working on task".to_string();
        start_timer(StartTimerArgs {
            duration: Some(30),
            message: message.clone(),
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.message, message);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_start_timer_with_add() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();
        thread::sleep(Duration::from_millis(100));

        start_timer(StartTimerArgs {
            add: Some(30),
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.duration, 90);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_start_timer_default_duration() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs::default()).unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.duration, DEFAULT_TIMER_DURATION);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_pause_timer() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();
        thread::sleep(Duration::from_millis(200));

        pause_timer().unwrap();
        thread::sleep(Duration::from_millis(50));

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Paused);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_pause_resume_timer() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();
        thread::sleep(Duration::from_millis(100));

        pause_timer().unwrap();
        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Paused);

        pause_timer().unwrap();
        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Running);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_stop_timer() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();
        stop_timer().unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Finished);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_get_status_paused() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();
        pause_timer().unwrap();
        let status = get_status(Some(StatusFormat::Human), Some(TimeFormat::Digital)).unwrap();

        assert!(status.contains("Paused"));

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_timer_with_flags() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            silent: true,
            notify: true,
            wait: true,
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert!(timer_info.silent);
        assert!(timer_info.notify);
        assert!(timer_info.wait);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_status_with_message() {
        let _temp = setup_test_env();

        let message = "Test message".to_string();
        start_timer(StartTimerArgs {
            duration: Some(60),
            message: message.clone(),
            ..Default::default()
        })
        .unwrap();

        let status = get_status(Some(StatusFormat::Human), Some(TimeFormat::Digital)).unwrap();
        assert!(status.contains("Test message"));

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_status_json_format() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(100),
            message: "JSON test".to_string(),
            ..Default::default()
        })
        .unwrap();

        let status = get_status(Some(StatusFormat::Json), Some(TimeFormat::Segmented)).unwrap();

        let json: serde_json::Value = serde_json::from_str(&status).unwrap();
        assert!(json.is_object());
        assert!(json.get("class").is_some());
        assert!(json.get("text").is_some());
        assert!(json.get("percentage").is_some());

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_status_finished_timer() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(10),
            ..Default::default()
        })
        .unwrap();

        stop_timer().unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Finished);

        let status = get_status(Some(StatusFormat::Human), Some(TimeFormat::Digital)).unwrap();
        assert!(status.contains("Time is up!"));

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_pause_when_not_running() {
        let _temp = setup_test_env();

        TimerInfo::remove_info_file().ok();

        let result = pause_timer();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_stop_when_not_running() {
        let _temp = setup_test_env();

        TimerInfo::remove_info_file().ok();

        let result = stop_timer();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_set_cycle_saves_phases() {
        let _temp = setup_test_env();

        let phases = vec!["Work:25".to_string(), "Break:5".to_string()];
        set_cycle(&phases).unwrap();

        let cycle = crate::cycle::CycleDefinition::load_or_default();
        assert_eq!(cycle.len(), 2);
        assert_eq!(cycle.phase(0).name, "Work");
        assert_eq!(cycle.phase(0).minutes, 25);
        assert_eq!(cycle.phase(1).name, "Break");
        assert_eq!(cycle.phase(1).minutes, 5);
    }

    #[test]
    #[serial]
    fn test_set_cycle_no_args_resets_to_default() {
        let _temp = setup_test_env();

        set_cycle(&[]).unwrap();

        let cycle = crate::cycle::CycleDefinition::load_or_default();
        assert_eq!(cycle.len(), 8);
    }

    #[test]
    #[serial]
    fn test_start_cycle_uses_first_phase() {
        let _temp = setup_test_env();

        let phases = vec!["Focus:30".to_string(), "Rest:10".to_string()];
        set_cycle(&phases).unwrap();

        start_timer(StartTimerArgs {
            cycle: true,
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Running);
        assert_eq!(timer_info.duration, 30 * 60);
        assert_eq!(timer_info.cycle_phase_index, Some(0));
        assert_eq!(timer_info.cycle_total, Some(2));
        assert_eq!(timer_info.cycle_phase_name, "Focus");
        assert!(!timer_info.repeat);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_start_cycle_with_repeat() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            cycle: true,
            repeat: true,
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert!(timer_info.repeat);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_advance_cycle_phase_moves_to_next() {
        let _temp = setup_test_env();

        let phases = vec!["Work:25".to_string(), "Break:5".to_string()];
        set_cycle(&phases).unwrap();

        start_timer(StartTimerArgs {
            cycle: true,
            ..Default::default()
        })
        .unwrap();

        let mut timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.cycle_phase_index, Some(0));

        let advanced = advance_cycle_phase(&mut timer_info).unwrap();
        assert!(advanced);
        assert_eq!(timer_info.cycle_phase_index, Some(1));
        assert_eq!(timer_info.cycle_phase_name, "Break");
        assert_eq!(timer_info.duration, 5 * 60);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_advance_cycle_phase_returns_false_at_end() {
        let _temp = setup_test_env();

        let phases = vec!["Work:25".to_string(), "Break:5".to_string()];
        set_cycle(&phases).unwrap();

        start_timer(StartTimerArgs {
            cycle: true,
            ..Default::default()
        })
        .unwrap();

        let mut timer_info = TimerInfo::from_file_or_default().unwrap();
        // Manually set to last phase
        timer_info.cycle_phase_index = Some(1);
        timer_info.write_to_file().unwrap();

        let advanced = advance_cycle_phase(&mut timer_info).unwrap();
        assert!(!advanced);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_advance_cycle_phase_repeats_when_repeat_true() {
        let _temp = setup_test_env();

        let phases = vec!["Work:25".to_string(), "Break:5".to_string()];
        set_cycle(&phases).unwrap();

        start_timer(StartTimerArgs {
            cycle: true,
            repeat: true,
            ..Default::default()
        })
        .unwrap();

        let mut timer_info = TimerInfo::from_file_or_default().unwrap();
        timer_info.cycle_phase_index = Some(1);
        timer_info.write_to_file().unwrap();

        let advanced = advance_cycle_phase(&mut timer_info).unwrap();
        assert!(advanced);
        assert_eq!(timer_info.cycle_phase_index, Some(0));
        assert_eq!(timer_info.cycle_phase_name, "Work");

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_repeat_flag_reuses_previous_duration() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(300),
            ..Default::default()
        })
        .unwrap();
        stop_timer().unwrap();

        start_timer(StartTimerArgs {
            repeat: true,
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Running);
        assert_eq!(timer_info.duration, 300);
        assert_eq!(timer_info.cycle_phase_index, None);
        assert!(timer_info.repeat);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_repeat_flag_no_previous_timer_uses_default_duration() {
        let _temp = setup_test_env();
        TimerInfo::remove_info_file().ok();

        start_timer(StartTimerArgs {
            repeat: true,
            ..Default::default()
        })
        .unwrap();

        let timer_info = TimerInfo::from_file_or_default().unwrap();
        assert_eq!(timer_info.state, TimerState::Running);
        assert_eq!(timer_info.duration, DEFAULT_TIMER_DURATION);
        assert!(timer_info.repeat);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_status_shows_cycle_label() {
        let _temp = setup_test_env();

        let phases = vec!["Deep Work:50".to_string(), "Rest:10".to_string()];
        set_cycle(&phases).unwrap();

        start_timer(StartTimerArgs {
            cycle: true,
            ..Default::default()
        })
        .unwrap();

        let status = get_status(Some(StatusFormat::Human), Some(TimeFormat::Digital)).unwrap();
        assert!(status.contains("[Deep Work 1/2]"), "status was: {}", status);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_status_json_includes_cycle_fields() {
        let _temp = setup_test_env();

        let phases = vec!["Work:25".to_string(), "Break:5".to_string()];
        set_cycle(&phases).unwrap();

        start_timer(StartTimerArgs {
            cycle: true,
            ..Default::default()
        })
        .unwrap();

        let status = get_status(Some(StatusFormat::Json), Some(TimeFormat::Digital)).unwrap();
        let json: serde_json::Value = serde_json::from_str(&status).unwrap();
        assert_eq!(json["cycle_phase"], "Work");
        assert_eq!(json["cycle_index"], 1);
        assert_eq!(json["cycle_total"], 2);

        let _ = TimerInfo::remove_info_file();
    }

    #[test]
    #[serial]
    fn test_plain_timer_status_has_no_cycle_fields() {
        let _temp = setup_test_env();

        start_timer(StartTimerArgs {
            duration: Some(60),
            ..Default::default()
        })
        .unwrap();

        let status = get_status(Some(StatusFormat::Json), Some(TimeFormat::Digital)).unwrap();
        let json: serde_json::Value = serde_json::from_str(&status).unwrap();
        assert!(json.get("cycle_phase").is_none());
        assert!(json.get("cycle_index").is_none());
        assert!(json.get("cycle_total").is_none());

        let _ = TimerInfo::remove_info_file();
    }
}

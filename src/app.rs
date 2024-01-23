use crate::args::*;
use crate::error::*;
use crate::timer_info::DEFAULT_TIMER_DURATION;
use crate::timer_info::{TimerInfo, TimerState};
use crate::timer_profile::TimerProfile;
use crate::utils::*;
use crossterm::cursor::{MoveToColumn, MoveToPreviousLine};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use notify_rust::Notification;
use rodio::{Decoder, Source};
use std::thread;

pub const DEFAULT_ALERT_FILE: &str = "alarm.mp3";
pub const DEFAULT_ICON_FILE: &str = "icon.png";

/// Run the application with the given arguments
pub fn run(args: &Cli) -> AppResult<()> {
    match &args.subcmd {
        SubCommand::New {
            profile,
            sequence,
            messages,
            repeat,
            alert_path,
            icon_path,
            notify,
            silent,
        } => {
            create_timer_profile(
                profile.clone(),
                sequence.clone(),
                messages.clone().unwrap_or_default(),
                repeat.unwrap_or_else(|| 1),
                alert_path.clone().unwrap_or_default(),
                icon_path.clone().unwrap_or_default(),
                *notify,
                *silent,
            )?;
        }
        SubCommand::Start {
            profile,
            duration,
            add,
            message,
            silent,
            notify,
            wait,
            resume,
        } => {
            if let Some(profile) = profile {
                println!("Starting timer with profile: {}", profile);
            } else {
                start_timer(
                    if let Ok(duration) = parse_duration(duration) {
                        Some(duration)
                    } else {
                        None
                    },
                    if let Ok(add) = parse_duration(add) {
                        Some(add)
                    } else {
                        None
                    },
                    message.clone().unwrap_or("".to_string()),
                    *silent,
                    *notify,
                    *resume,
                )?;
            }
            if *wait {
                wait_for_timer()?;
            }
        }
        SubCommand::Pause => {
            pause_timer()?;
        }
        SubCommand::Stop => {
            stop_timer()?;
        }
        SubCommand::Status { format } => {
            let status = get_status(*format)?;
            println!("{}", status);
        }
    }
    Ok(())
}

/// Start the timer. If the timer is already running, the duration is added to the current duration.
pub fn start_timer(
    duration: Option<i64>,
    add: Option<i64>,
    message: String,
    silent: bool,
    notify: bool,
    resume: bool,
) -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    if timer_info.is_running() && add.is_some() {
        // Add more time to the timer
        timer_info.duration += add.unwrap();
    } else if timer_info.is_paused() && resume {
        // Resume a paused timer
        let now = chrono::Utc::now().timestamp();
        let elapsed = timer_info.pause_time - timer_info.start_time;
        timer_info.duration = timer_info.duration - elapsed;
        timer_info.start_time = now;
        timer_info.pause_time = now;
        timer_info.message = timer_info.message.clone();
        timer_info.silent = timer_info.silent || silent;
        timer_info.notify = timer_info.notify || notify;
        timer_info.state = TimerState::Running;
    } else {
        // Start a new timer
        let duration = duration.unwrap_or(add.unwrap_or(DEFAULT_TIMER_DURATION));
        let now = chrono::Utc::now().timestamp() + 1;
        timer_info.duration = duration;
        timer_info.start_time = now;
        timer_info.pause_time = now;
        timer_info.message = message;
        timer_info.silent = silent;
        timer_info.notify = notify;
        timer_info.state = TimerState::Running;
    }
    timer_info.write_to_file()?;
    Ok(())
}

/// Pause the timer. If the timer is already paused, the timer is resumed.
pub fn pause_timer() -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    if timer_info.is_running() {
        let now = chrono::Utc::now().timestamp();
        timer_info.pause_time = now;
        timer_info.state = TimerState::Paused;
        timer_info.write_to_file()?;
    } else {
        start_timer(
            Some(timer_info.duration),
            None,
            timer_info.message,
            timer_info.silent,
            timer_info.notify,
            false,
        )?;
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

/// Trigger the alarm sound and/or the system notification.
pub fn trigger_alarm(timer_info: &TimerInfo) -> AppResult<()> {
    println!("Time is up!");

    if timer_info.notify {
        let mut path = String::from("dialog-warning");
        if let Ok(custom_icon_path) = create_config_file_path(DEFAULT_ICON_FILE) {
            if custom_icon_path.exists() {
                path = custom_icon_path.to_str().unwrap_or(&path).to_string();
            }
        }
        Notification::new()
            .summary("Pomodoro Timer")
            .body("Time is up!")
            .icon(&path)
            .appname("pomodoro-cli")
            .show()?;
    }

    if !timer_info.silent {
        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let custom_alert_path = create_config_file_path(DEFAULT_ALERT_FILE)?;
        if custom_alert_path.exists() {
            println!("Playing alert sound from: {:?}", custom_alert_path);
            let file = std::fs::File::open(custom_alert_path)?;
            let source = Decoder::new(file)?;
            stream_handle.play_raw(source.convert_samples()).unwrap();
        } else {
            println!("Playing default alert sound.");
            let mp3 = include_bytes!("../assets/ding.mp3");
            let cursor = std::io::Cursor::new(mp3);
            let source = Decoder::new_mp3(cursor)?;
            stream_handle.play_raw(source.convert_samples()).unwrap();
        }
        // TODO: This is a hack to keep the thread alive until the sound is played.
        std::thread::sleep(std::time::Duration::from_millis(2000));
        println!("Alert sound played.");
    }
    return Ok(());
}

/// Return the status of the timer in the given format.
pub fn get_status(format: Option<StatusFormat>) -> AppResult<String> {
    let timer_info = TimerInfo::from_file_or_default()?;
    let status: String = match format {
        Some(StatusFormat::Human) => format!("{}", timer_info.get_human_readable()),
        Some(StatusFormat::Json) => format!("{}", timer_info.get_json_info()?),
        _ => format!("{}", timer_info.get_time_left()),
    };

    if timer_info.is_running() && !timer_info.wait && timer_info.is_time_run_out() {
        stop_timer()?;
        trigger_alarm(&timer_info)?;
    }
    Ok(status)
}

/// Wait for the timer to finish.
pub fn wait_for_timer() -> AppResult<()> {
    // This thread will wait for the timer to finish and peridoically prints the time left.
    let timer_thrd = thread::spawn(move || -> AppResult<()> {
        let mut stdout = std::io::stdout();
        loop {
            let timer_info = TimerInfo::from_file_or_default()?;
            let percentage = (timer_info.get_percentage() / 4.0) as i64;
            print!("|");
            for _ in 0..percentage {
                print!("#");
            }
            for _ in 0..(25 - percentage) {
                print!("-");
            }
            println!("| {}", timer_info.get_human_readable());

            thread::sleep(std::time::Duration::from_millis(1000));
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
                stop_timer()?;
                trigger_alarm(&timer_info)?;
                break;
            }
        }
        return Ok(());
    });

    if let Err(e) = timer_thrd.join() {
        return Err(AppError::new(&format!("Error: {:?}", e)));
    }
    return Ok(());
}

/// Create a new timer profile
fn create_timer_profile(
    profile: String,
    sequence: Vec<String>,
    messages: Vec<String>,
    repeat: u32,
    alert_path: String,
    icon_path: String,
    notify: bool,
    silent: bool,
) -> AppResult<()> {
    let sequence = sequence
        .iter()
        .map(|s| {
            parse_duration(&Some(s.to_string())).unwrap_or_else(|e| {
                eprintln!("{}", e);
                return 0;
            })
        })
        .collect::<Vec<i64>>();

    let timer_profile = TimerProfile {
        name: profile,
        sequence,
        messages,
        repeat,
        alert_path,
        icon_path,
        silent,
        notify,
    };
    timer_profile.write_to_file()?;
    Ok(())
}

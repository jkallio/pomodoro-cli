use crate::args::*;
use crate::error::*;
use crate::timer_info::DEFAULT_TIMER_DURATION;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use crossterm::cursor::{MoveToColumn, MoveToPreviousLine};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use lock::FailureReason;
use notify_rust::{Notification, Timeout};
use rodio::{Decoder, OutputStream, Sink};
use std::thread;
use std::time::Duration;

/// Run the application with the given arguments
pub fn run(args: &Cli) -> AppResult<()> {
    match &args.subcmd {
        SubCommand::Start {
            duration,
            add,
            message,
            silent,
            notify,
            wait,
            resume,
            lock_screen,
        } => {
            start_timer(
                parse_duration(duration.clone()),
                parse_duration(add.clone()),
                message.clone().unwrap_or("".to_string()),
                *silent,
                *notify,
                *resume,
                *lock_screen,
            )?;
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
        } => {
            let status = get_status(*format, *time_format)?;
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
    lock_screen: bool,
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
        timer_info.lock_screen = timer_info.lock_screen || lock_screen;
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
        timer_info.lock_screen = lock_screen;
    }
    timer_info.write_to_file()?;
    Ok(())
}

/// Pause the timer. If the timer is already paused, the timer is resumed.
pub fn pause_timer() -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    if timer_info.is_paused() {
        start_timer(
            Some(timer_info.duration),
            None,
            timer_info.message,
            timer_info.silent,
            timer_info.notify,
            true,
            timer_info.lock_screen,
        )?;
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
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle).unwrap();
        if let Some(path) = get_custom_alarm_file() {
            let file = std::fs::File::open(path)?;
            let source = Decoder::new(file)?;
            sink.append(source);
        } else {
            let mp3 = include_bytes!("../assets/ding.mp3");
            let source = Decoder::new(std::io::Cursor::new(mp3))?;
            sink.append(source);
        }
        sink.set_volume(1.0);
        sink.sleep_until_end();
        sink.clear();
    }

    // Now check if the lock screen option is enabled
    if timer_info.lock_screen {
        lock_screen()?;
    }

    return Ok(());
}

/// Return the status of the timer in the given format.
pub fn get_status(
    format: Option<StatusFormat>,
    time_format: Option<TimeFormat>,
) -> AppResult<String> {
    let timer_info = TimerInfo::from_file_or_default()?;
    let status: String = match format {
        Some(StatusFormat::Json) => {
            format!(
                "{}",
                timer_info.get_json_info(time_format.unwrap_or_default())?
            )
        }
        _ => {
            format!(
                "{}",
                timer_info.get_human_readable(time_format.unwrap_or_default())
            )
        }
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
            println!("| {}", timer_info.get_human_readable(TimeFormat::default()));

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

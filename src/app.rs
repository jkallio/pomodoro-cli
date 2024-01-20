use crate::args::*;
use crate::error::*;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use crossterm::cursor::{Hide, MoveToColumn, MoveToPreviousLine, Show};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use notify_rust::Notification;
use rodio::{Decoder, Source};
use std::thread;

/// Run the application with the given arguments
pub fn run(args: &Cli) -> AppResult<()> {
    match &args.subcmd {
        SubCommand::Start {
            duration,
            silent,
            notify,
            wait,
            add,
        } => {
            start_timer(parse_duration(duration.clone()), *silent, *notify, *add)?;
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
pub fn start_timer(duration: i64, silent: bool, notify: bool, add: bool) -> AppResult<()> {
    let mut timer_info = TimerInfo::from_file_or_default()?;
    if add && timer_info.is_running() {
        timer_info.duration += duration;
    } else {
        let now = chrono::Utc::now().timestamp() + 1;
        let elapsed = timer_info.pause_time - timer_info.start_time;
        timer_info.duration = duration - elapsed;
        timer_info.start_time = now;
        timer_info.pause_time = now;
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
            timer_info.duration,
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
        if let Some(custom_icon_path) = get_custom_icon_file() {
            path = custom_icon_path.to_str().unwrap_or(&path).to_string();
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
        if let Some(path) = get_custom_alarm_file() {
            let file = std::fs::File::open(path)?;
            let source = Decoder::new(file)?;
            stream_handle.play_raw(source.convert_samples()).unwrap();
        } else {
            let mp3 = include_bytes!("../assets/ding.mp3");
            let cursor = std::io::Cursor::new(mp3);
            let source = Decoder::new_mp3(cursor)?;
            stream_handle.play_raw(source.convert_samples()).unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
    return Ok(());
}

/// Return the status of the timer in the given format.
pub fn get_status(format: Option<StatusFormat>) -> AppResult<String> {
    let timer_info = TimerInfo::from_file_or_default()?;
    let status: String = match format {
        Some(StatusFormat::Human) => match timer_info.state {
            TimerState::Finished => "Finished".to_string(),
            TimerState::Paused => format!(
                "Paused ({} left)",
                get_human_readable_time(timer_info.get_time_left())
            ),
            TimerState::Running => {
                format!("{}", get_human_readable_time(timer_info.get_time_left()))
            }
        },
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
        execute!(stdout, Hide)?;
        loop {
            let timer_info = TimerInfo::from_file_or_default()?;
            println!("{}", get_human_readable_time(timer_info.get_time_left()));

            thread::sleep(std::time::Duration::from_millis(1000));
            execute!(
                stdout,
                MoveToPreviousLine(1),
                MoveToColumn(0),
                Clear(ClearType::CurrentLine)
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
        execute!(stdout, Show)?;
        return Ok(());
    });

    if let Err(e) = timer_thrd.join() {
        return Err(AppError::new(&format!("Error: {:?}", e)));
    }
    return Ok(());
}

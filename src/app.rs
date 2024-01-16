use crate::args::*;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use crossterm::cursor::{Hide, MoveToColumn, MoveToPreviousLine, Show};
use crossterm::event::{self, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType};
use notify_rust::Notification;
use rodio::{Decoder, Source};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Run the application with the given arguments
pub fn run(args: &Cli) {
    match &args.subcmd {
        SubCommand::Start {
            duration,
            silent,
            notify,
            wait,
        } => {
            start_timer(parse_duration(duration.clone()), *silent, *notify);
            if *wait {
                wait_for_timer();
            }
        }
        SubCommand::Pause => {
            pause_timer();
        }
        SubCommand::Stop => {
            stop_timer();
        }
        SubCommand::Status { format } => {
            let status = get_status(*format);
            println!("{}", status);
        }
    }
}

/// Start the timer. If the timer is already running, the duration is added to the current duration.
pub fn start_timer(duration: i64, silent: bool, notify: bool) {
    let mut timer_info = TimerInfo::from_file();
    if timer_info.is_running() {
        timer_info.duration += duration;
    } else {
        let elapsed = timer_info.pause_time - timer_info.start_time;
        timer_info.duration = duration - elapsed;
        timer_info.start_time = chrono::Utc::now().timestamp();
        timer_info.pause_time = timer_info.start_time;
        timer_info.silent = silent;
        timer_info.notify = notify;
        timer_info.state = TimerState::Running;
    }
    timer_info.write_to_file();
}

/// Pause the timer. If the timer is already paused, the timer is resumed.
pub fn pause_timer() {
    let mut timer_info = TimerInfo::from_file();
    if timer_info.is_running() {
        let now = chrono::Utc::now().timestamp();
        timer_info.pause_time = now;
        timer_info.state = TimerState::Paused;
        timer_info.write_to_file();
    } else {
        start_timer(timer_info.duration, timer_info.silent, timer_info.notify);
    }
}

/// Stop the timer.
pub fn stop_timer() {
    let mut timer_info = TimerInfo::from_file();
    timer_info.state = TimerState::Finished;
    timer_info.write_to_file();
}

/// Trigger the alarm sound and/or the system notification.
pub fn trigger_alarm(timer_info: &TimerInfo) {
    println!("Time is up!");

    if timer_info.notify {
        let mut path = String::from("dialog-warning");
        if let Some(custom_icon_path) = get_custom_icon_file() {
            if let Some(custom_icon_path) = custom_icon_path.to_str() {
                path = custom_icon_path.to_string();
            }
        }

        Notification::new()
            .summary("Pomodoro Timer")
            .body("Time is up!")
            .icon(&path)
            .appname("pomodoro-cli")
            .show()
            .unwrap();
    }

    if !timer_info.silent {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        if let Some(path) = get_custom_alarm_file() {
            let file = std::fs::File::open(path).unwrap();
            let source = Decoder::new(file).unwrap();
            stream_handle.play_raw(source.convert_samples()).unwrap();
        } else {
            let mp3 = include_bytes!("../assets/ding.mp3");
            let cursor = std::io::Cursor::new(mp3);
            let source = Decoder::new_mp3(cursor).unwrap();
            stream_handle.play_raw(source.convert_samples()).unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}

/// Return the status of the timer in the given format.
pub fn get_status(format: Option<StatusFormat>) -> String {
    let timer_info = TimerInfo::from_file();

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
        Some(StatusFormat::Json) => format!("{}", timer_info.get_json_info()),
        _ => format!("{}", timer_info.get_time_left()),
    };

    if timer_info.is_running() && timer_info.is_time_run_out() {
        stop_timer();
        trigger_alarm(&timer_info);
    }
    return status;
}

/// Wait for the timer to finish.
pub fn wait_for_timer() {
    let (tx_interrupt, rx_interrupt) = mpsc::channel::<bool>();
    let (tx_timer_finished, rx_timer_finished) = mpsc::channel::<bool>();

    // This thread will wait for the timer to finish and peridoically prints the time left.
    let waiter = thread::spawn(move || {
        let mut stdout = std::io::stdout();
        execute!(stdout, Hide).unwrap();
        loop {
            let timer_info = TimerInfo::from_file();
            println!("{}", get_human_readable_time(timer_info.get_time_left()));

            let rx_interrupt = rx_interrupt.recv_timeout(Duration::from_secs(1));
            execute!(
                stdout,
                MoveToPreviousLine(1),
                MoveToColumn(0),
                Clear(ClearType::CurrentLine)
            )
            .unwrap();

            if !timer_info.is_running() || rx_interrupt.is_ok() {
                println!("Timer interrupted!");
                stop_timer();
                break;
            }

            if timer_info.is_time_run_out() {
                stop_timer();
                trigger_alarm(&timer_info);
                break;
            }
        }
        execute!(stdout, Show).unwrap();
        if tx_timer_finished.send(true).is_ok() {
            println!("Signaled timer finished!");
        }
    });

    // This thread waits for the user to press <Esc> or <q> to interrupt the timer.
    let keyboard_interrupt = thread::spawn(move || {
        terminal::enable_raw_mode().unwrap();
        loop {
            if rx_timer_finished.try_recv().is_ok() {
                break;
            }
            if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
                if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
                    match code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            if tx_interrupt.send(true).is_ok() {
                                println!("Signaled keyboard interrupt!");
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        terminal::disable_raw_mode().unwrap();
    });
    waiter.join().unwrap();
    keyboard_interrupt.join().unwrap();
    println!();
}

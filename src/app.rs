use crate::args::*;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use notify_rust::Notification;
use rodio::{Decoder, Source};

pub fn run(args: &Cli) {
    match &args.subcmd {
        SubCommand::Start {
            duration,
            silent,
            notify,
        } => {
            start_timer(parse_duration(duration.clone()), *silent, *notify);
        }
        SubCommand::Pause => {
            pause_timer();
        }
        SubCommand::Stop => {
            stop_timer();
        }
        SubCommand::Status { format } => {
            let format = match *format {
                Some(StatusFormat::Human) => StatusFormat::Human,
                _ => StatusFormat::Seconds,
            };
            get_status(format);
        }
    }
}

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

pub fn stop_timer() {
    let mut timer_info = TimerInfo::from_file();
    timer_info.state = TimerState::Finished;
    timer_info.write_to_file();
}

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
            println!("Playing custom alarm...");
            let file = std::fs::File::open(path).unwrap();
            let source = Decoder::new(file).unwrap();
            stream_handle.play_raw(source.convert_samples()).unwrap();
        } else {
            println!("Playing alarm...");
            let mp3 = include_bytes!("../assets/alarm.mp3");
            let cursor = std::io::Cursor::new(mp3);
            let source = Decoder::new_mp3(cursor).unwrap();
            stream_handle.play_raw(source.convert_samples()).unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(3000));
    }
}

pub fn get_status(format: StatusFormat) {
    let timer_info = TimerInfo::from_file();
    let elapsed = timer_info.get_time_elapsed();

    match timer_info.state {
        TimerState::Finished => {
            println!("Finished");
        }
        TimerState::Paused => {
            println!(
                "Paused ({} left)",
                get_human_readable_time(timer_info.get_time_left())
            );
            return;
        }
        TimerState::Running => {
            if elapsed >= timer_info.duration {
                stop_timer();
                trigger_alarm(&timer_info);
                return;
            }
            match format {
                StatusFormat::Human => {
                    println!("{}", get_human_readable_time(timer_info.get_time_left()))
                }
                StatusFormat::Seconds => {
                    println!("{}", timer_info.get_time_left());
                }
            }
        }
    }
}

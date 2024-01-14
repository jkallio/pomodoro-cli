use crate::args::*;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use notify_rust::Notification;
use rodio::{Decoder, Source};

pub fn run(args: &Cli) {
    match &args.subcmd {
        SubCommand::Reset {
            duration,
            silent,
            default,
        } => {
            reset_timer(parse_duration(duration.clone()), *silent, *default);
        }
        SubCommand::Start { wait } => {
            start_timer(*wait);
        }
        SubCommand::Stop => {
            stop_timer();
        }
        SubCommand::Toggle { wait } => {
            toggle_timer(*wait);
        }
        SubCommand::Status { format } => {
            let format = match *format {
                Some(StatusFormat::Human) => StatusFormat::Human,
                _ => StatusFormat::Seconds,
            };
            get_status(format);
        }
        SubCommand::Add { duration } => {
            add_time_to_timer(parse_duration(duration.clone()));
        }
    }
}

pub fn reset_timer(duration: i64, silent: bool, default: bool) {
    let timer_info = if default {
        TimerInfo::default()
    } else {
        let mut timer_info = TimerInfo::from_file();
        timer_info.duration = duration;
        timer_info.silent = silent;
        timer_info
    };
    timer_info.write_to_file();
}

pub fn start_timer(wait: bool) {
    let mut timer_info = TimerInfo::from_file();
    timer_info.start_time = chrono::Utc::now().timestamp();
    timer_info.state = TimerState::Running;
    timer_info.write_to_file();
}

pub fn stop_timer() {
    let mut timer_info = TimerInfo::from_file();
    let now = chrono::Utc::now().timestamp();
    let elapsed = now - timer_info.start_time;
    timer_info.duration -= elapsed;
    timer_info.state = TimerState::Stopped;
    timer_info.write_to_file();
}

pub fn toggle_timer(wait: bool) {
    let mut timer_info = TimerInfo::from_file();
    match timer_info.state {
        TimerState::Stopped => {
            start_timer(wait);
        }
        TimerState::Running => {
            let now = chrono::Utc::now().timestamp();
            let elapsed = now - timer_info.start_time;
            timer_info.duration -= elapsed;
            timer_info.write_to_file();
            stop_timer();
        }
    }
}

pub fn trigger_notification(timer_info: &TimerInfo) {
    println!("Time is up!");
    if timer_info.silent {
        return;
    }

    let mut path = String::from("warning");
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
    trigger_audio_alarm();
}

pub fn get_status(format: StatusFormat) {
    let timer_info = TimerInfo::from_file();
    let elapsed = timer_info.get_time_elapsed();

    match timer_info.state {
        TimerState::Stopped => {
            if timer_info.is_finished() {
                println!("Finished");
                return;
            }
            println!("Stopped");
            return;
        }
        TimerState::Running => {
            if timer_info.is_finished() {
                stop_timer();
                trigger_notification(&timer_info);
                return;
            }
            let remaining = timer_info.duration - elapsed;
            match format {
                StatusFormat::Human => {
                    println!("{}", get_human_readable_time(remaining))
                }
                StatusFormat::Seconds => {
                    println!("{}", remaining);
                }
            }
        }
    }
}

pub fn add_time_to_timer(duration: i64) {
    let mut timer_info = TimerInfo::from_file();
    timer_info.duration += duration;
    timer_info.write_to_file();
}

pub fn trigger_audio_alarm() {
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

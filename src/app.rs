use crate::args::*;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::*;
use notify_rust::Notification;

pub fn run(args: &Args) {
    match args.subcmd {
        SubCommand::Reset => {
            reset_timer(&args);
        }
        SubCommand::Start => {
            if args.toggle {
                toggle_start_stop();
            } else {
                start_timer();
            }
        }
        SubCommand::Stop => {
            stop_timer();
        }
        SubCommand::Status => {
            status(&args);
        }
        SubCommand::Add => {
            add_time(&args);
        }
    }
}

pub fn reset_timer(args: &Args) {
    let timer_info = TimerInfo::from_args(args);
    timer_info.write_to_file();
}

pub fn start_timer() {
    let mut timer_info = TimerInfo::from_file();
    timer_info.start_time = chrono::Utc::now().timestamp();
    timer_info.state = TimerState::Running;
    timer_info.write_to_file();
}

pub fn stop_timer() {
    let mut timer_info = TimerInfo::from_file();
    timer_info.state = TimerState::Stopped;
    timer_info.write_to_file();
}

pub fn toggle_start_stop() {
    let mut timer_info = TimerInfo::from_file();
    match timer_info.state {
        TimerState::Stopped => {
            start_timer();
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

pub fn status(args: &Args) {
    let timer_info = TimerInfo::from_file();
    let elapsed = timer_info.get_time_elapsed();

    match timer_info.state {
        TimerState::Stopped => {
            println!("Stopped");
            return;
        }
        TimerState::Running => {
            if timer_info.is_finished() {
                trigger_alarm(&timer_info);
                stop_timer();
                return;
            }
            let remaining = timer_info.duration - elapsed;
            match args.format {
                Some(StatusFormat::Human) => println!("{}", get_human_readable_time(remaining)),
                _ => {
                    println!("{}", remaining);
                }
            }
        }
    }
}

pub fn add_time(args: &Args) {
    if let Some(duration) = &args.duration {
        let mut timer_info = TimerInfo::from_file();
        timer_info.duration += parse_duration(duration);
        timer_info.write_to_file();
    }
}

pub fn trigger_alarm(timer_info: &TimerInfo) {
    println!("Time is up!");
    if timer_info.silent {
        return;
    }
    Notification::new()
        .summary("Pomodoro Timer")
        .body("Time is up!")
        .icon("dialog-warning")
        .appname("pomodoro-cli")
        .show()
        .unwrap();
}

use crate::args::*;
use crate::timer_info::{TimerInfo, TimerState};
use crate::utils::get_human_readable_time;
use notify_rust::Notification;

pub fn run(args: &Args) {
    match args.subcmd {
        SubCommand::Reset => {
            reset_timer(&args);
        }
        SubCommand::Start => {
            start_timer(&args);
        }
        SubCommand::Stop => {
            stop_timer();
        }
        SubCommand::Status => {
            status(&args);
        }
    }
}

pub fn reset_timer(args: &Args) {
    let timer_info = TimerInfo::from_args(args);
    timer_info.write_to_file();
}

pub fn start_timer(args: &Args) {
    let mut timer_info = TimerInfo::from_file();
    if timer_info.is_finished() {
        println!("Timer is already finished. Reset it first.");
        return;
    } else if timer_info.state == TimerState::Running && !args.toggle {
        println!("Timer is already running.");
        return;
    }
    let now = chrono::Utc::now().timestamp() as u64;
    let elapsed_during_pause: i64 = (now - timer_info.pause_time) as i64;
    timer_info.start_time += elapsed_during_pause as u64;
    timer_info.pause_time = timer_info.start_time;
    timer_info.state = TimerState::Running;
    timer_info.write_to_file();
}

pub fn stop_timer() {
    let mut timer_info = TimerInfo::from_file();
    timer_info.pause_time = chrono::Utc::now().timestamp() as u64;
    timer_info.state = TimerState::Stopped;
    timer_info.write_to_file();
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
            let remaining = timer_info.duration - elapsed as u64;
            match args.format {
                Some(StatusFormat::Human) => println!("{}", get_human_readable_time(remaining)),
                _ => {
                    println!("{}", remaining);
                }
            }
        }
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

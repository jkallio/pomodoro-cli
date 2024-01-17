mod app;
mod args;
mod error;
mod timer_info;
mod utils;

use crate::args::Cli;
use clap::Parser;

/// Pomodoro timer is a simple timer that helps you to stay focused on your task.
fn main() {
    let args = Cli::parse();
    if let Err(e) = app::run(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

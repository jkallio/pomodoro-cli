mod app;
mod args;
mod timer_info;
mod utils;

use crate::args::Cli;
use clap::Parser;

/// Pomodoro timer is a simple timer that helps you to stay focused on your task.
fn main() {
    let args = Cli::parse();
    app::run(&args);
}

mod app;
mod args;
mod timer_info;
mod utils;

use crate::args::Cli;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    app::run(&args);
}

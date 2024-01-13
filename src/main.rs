mod app;
mod args;
mod timer_info;
mod utils;

use crate::args::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();
    app::run(&args);
}

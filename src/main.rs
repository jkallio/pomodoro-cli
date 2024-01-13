mod app;
use clap::Parser;

fn main() {
    let args = app::Args::parse();
    app::run(&args);
}

use clap::{Parser, Subcommand};
use std::path::PathBuf;

const STATE_FILE: &str = "pomodoro-cli-status.json";

#[derive(Parser, Debug)]
#[clap(
    name = "Pomodoro CLI",
    version = "0.1.0",
    author = "Jussi Kallio",
    about = "Pomodoro timer is a simple timer that helps you to stay focused on your task."
)]
pub struct Args {
    #[clap(subcommand)]
    pub subcmd: SubCommand,

    #[clap(short, long, global = true)]
    pub duration: Option<String>,

    #[clap(short, long, global = true)]
    pub task: Option<String>,

    #[arg(short, long, default_value_t = false, global = true)]
    pub wait: bool,

    #[clap(short, long, default_value_t = false, global = true)]
    pub alarm: bool,

    #[clap(short, long, global = true)]
    pub notify: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Start,
    Stop,
    Restart,
    Status,
}

pub fn run(args: &Args) {
    let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(STATE_FILE);

    println!("Args: {:?}", args);
}

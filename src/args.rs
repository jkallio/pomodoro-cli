use clap::{Parser, Subcommand};
use serde::Serialize;

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

    #[arg(short, long, default_value_t = false, global = true)]
    pub wait: bool,

    #[clap(short, long, default_value_t = false, global = true)]
    pub silent: bool,

    #[clap(short, long, global = true)]
    pub format: Option<StatusFormat>,

    #[clap(long, default_value_t = false, global = true)]
    pub default: bool,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Reset,
    Start,
    Stop,
    Toggle,
    Status,
    Add,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusFormat {
    #[default]
    Seconds,
    Human,
}

use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser, Debug)]
#[clap(
    name = "Pomodoro CLI",
    version = "1.0.0",
    author = "Jussi Kallio",
    about = "Pomodoro timer is a simple timer that helps you to stay focused on your task."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Start a new timer
    Start {
        #[arg(short, long, help = "Duration of the timer in format 1h30m20s")]
        duration: Option<String>,

        #[arg(long, default_value_t = false, help = "Enable system notification")]
        notify: bool,

        #[arg(long, default_value_t = false, help = "Disable the alarm sound")]
        silent: bool,
    },
    /// Stop the timer
    Stop,
    /// Pause/Resume the timer
    Pause,
    /// Get the current status of the timer
    Status {
        #[arg(
            short,
            long,
            help = "Duration in seconds or human-readable format (default=seconds)"
        )]
        format: Option<StatusFormat>,
    },
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusFormat {
    #[default]
    Seconds,
    Human,
}

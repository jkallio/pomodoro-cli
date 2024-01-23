use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

/// Defines the subcommands for the CLI
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Create a new timer profile
    New {
        #[arg(short, long, help = "Timer profile name")]
        profile: String,

        #[arg(
            short,
            long,
            help = "Timer sequence in format 1h 30m 20s",
            required = true,
            num_args = 1..
        )]
        sequence: Vec<String>,

        #[arg(
            short,
            long,
            help = "Timer messages",
            required = true,
            num_args = 1..
        )]
        messages: Option<Vec<String>>,

        #[arg(long, help = "Repeat the timer sequence")]
        repeat: Option<u32>,

        #[arg(short, long, help = "Path to the alarm sound")]
        alert_path: Option<String>,

        #[arg(short, long, help = "Path to the notification icon")]
        icon_path: Option<String>,

        #[arg(long, default_value_t = false, help = "Enable system notification")]
        notify: bool,

        #[arg(long, default_value_t = true, help = "Disable the alarm sound")]
        silent: bool,
    },
    /// Start a new timer
    Start {
        #[arg(
            short,
            long,
            help = "Timer profile name",
            conflicts_with_all = &["duration", "add", "resume", "message", "notify", "silent", "wait"],
        )]
        profile: Option<String>,

        #[arg(
            short,
            long,
            conflicts_with_all = &["add", "resume"],
            help = "Duration of the timer in format 1h 30m 20s"
        )]
        duration: Option<String>,

        #[arg(
            short,
            long,
            conflicts_with = "resume",
            help = "Add more time to the timer"
        )]
        add: Option<String>,

        #[arg(short, long, conflicts_with = "resume", help = "Timer message")]
        message: Option<String>,

        #[arg(long, default_value_t = false, help = "Enable system notification")]
        notify: bool,

        #[arg(long, default_value_t = false, help = "Disable the alarm sound")]
        silent: bool,

        #[arg(long, default_value_t = false, help = "Wait for the timer to finish")]
        wait: bool,

        #[arg(long, default_value_t = false, help = "Resume paused timer")]
        resume: bool,
    },
    /// Stop the timer
    Stop,
    /// Pause/Resume the timer
    Pause,
    /// Get the current status of the timer
    Status {
        #[arg(short, long, help = "Status format [seconds/human-readable/JSON]")]
        format: Option<StatusFormat>,
    },
}

/// Defines the returned time format for the status command
#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize, Copy)]
#[serde(rename_all = "lowercase")]
pub enum StatusFormat {
    #[default]
    Seconds,
    Human,
    Json,
}

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
    /// Start a new timer
    Start {
        #[arg(
            short,
            long,
            conflicts_with_all = &["add", "resume"],
            help = "Duration of the timer ('10m 30s' or '10:30')"
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

        #[arg(
            long,
            default_value_t = false,
            help = "Lock the screen when the timer finishes"
        )]
        lock_screen: bool,

        #[arg(
            long,
            default_value_t = false,
            conflicts_with = "duration",
            help = "Run the saved Pomodoro cycle instead of a single timer"
        )]
        cycle: bool,

        #[arg(
            long,
            default_value_t = false,
            help = "Repeat the previous timer, or the cycle"
        )]
        repeat: bool,
    },
    /// Stop the timer
    Stop,
    /// Pause/Resume the timer
    Pause,
    /// Define a custom Pomodoro cycle (stored in ~/.config/pomodoro-cli/cycle.json)
    SetCycle {
        #[arg(
            value_name = "NAME:MINUTES",
            help = "Phase definitions as 'Name:minutes' pairs (e.g. 'Work:25' 'Break:5'). \
                    No arguments resets to the built-in default (4×25 min work + short/long breaks)."
        )]
        phases: Vec<String>,
    },
    /// Get the current status of the timer
    Status {
        #[arg(short, long, help = "Status format")]
        format: Option<StatusFormat>,

        #[arg(short, long, help = "Time format")]
        time_format: Option<TimeFormat>,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Stream status continuously (one JSON line per second, for Waybar exec without interval)"
        )]
        watch: bool,
    },
}

/// Defines the returned time format for the status command
#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize, Copy)]
#[serde(rename_all = "lowercase")]
pub enum StatusFormat {
    #[default]
    Human,
    Json,
}

/// Defines the time format for the status command
#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    #[default]
    Digital, // 10:30
    Segmented, // 1h 10m 30s
    Seconds,   // 630
}

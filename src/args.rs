use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser, Debug)]
#[clap(
    name = "Pomodoro CLI",
    version = "0.9.0",
    author = "Jussi Kallio",
    about = "Pomodoro timer is a simple timer that helps you to stay focused on your task."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Reset {
        #[arg(short, long)]
        duration: Option<String>,

        #[arg(long, default_value_t = false)]
        silent: bool,

        #[arg(long, default_value_t = false)]
        default: bool,
    },
    Start {
        #[arg(long, default_value_t = false)]
        wait: bool,
    },
    Stop,
    Toggle {
        #[arg(long, default_value_t = false)]
        wait: bool,
    },
    Status {
        #[arg(short, long)]
        format: Option<StatusFormat>,
    },
    Add {
        #[arg(short, long)]
        duration: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusFormat {
    #[default]
    Seconds,
    Human,
}

<div align="center">

#  pomodoro-cli
# Pomodoro Timer Command Line Interface

Pomodoro timer is a simple timer that helps you to stay focused on your tasks.

`pomodoro-cli` is a CLI application which implements the basic functionalities of a basic Pomodoro timer. This application was designed to be used with [waybar](https://github.com/Alexays/Waybar).

[![Rust](https://img.shields.io/badge/Rust-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/pomodoro-cli.svg)](https://crates.io/crates/pomodoro-cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

</div>

# Installation

TODO

## Compilation from source

Pre-requisites:
- [Rust compilation tools](https://www.rust-lang.org/)

```bash
$ git clone git@github.com:jkallio/pomodoro-cli.git 
$ cd pomodoro-cli
$ cargo build --release
$ cp target/release/pomodoro-cli /usr/local/bin/pomodoro-cli
```

# Features

- [x] Start/Stop the Timer
- [x] Reset the Timer with custom configuration
- [x] Wait for the Timer to finish  
- [x] Query the Timer status
- [x] Triggers system notification when the Timer is finished
- [x] Play alarm sound when the Timer is finished
- [x] Easy Waybar integration
- [x] Customize notification icon
- [x] Customize alarm sound

# Usage

### Configure the timer

```bash
# Reset the timer with the default values
$ pomodoro-cli reset --default

# Reset the timer with custom values
$ pomodoro-cli reset --duration 25m --silent true --wait false
```

### Start/Stop the timer

```bash
# Start the timer
$ pomodoro-cli start

# Stop the timer
$ pomodoro-cli stop

# Toggle the timer (start/stop)
$ pomodoro-cli toggle
```

### Query the timer status

```bash
# Get remaining time in seconds (This is the default behavior for `status`)
$ pomodoro-cli status --format seconds

## Get remaining time in human readable format
$ pomodoro-cli status --format human
```

### Wait for the timer to finish

```bash
# Wait until the timer is finished
$ pomodoro-cli start --wait
```

# Customization

## Set custom alarm sound

If you want to use a custom alarm sound, just add a `alarm.mp3` file in the `~/.config/pomodoro-cli` directory.

```bash
$ mkdir -p ~/.config/pomodoro-cli
$ cp /path/to/alarm.mp3 ~/.config/pomodoro-cli/alarm.mp3
```

## Set custom notification icon 

![Waybar](./assets/screenshot_notification.png)

If you want to use a custom notification icon, just add a `icon.png` file in the `~/.config/pomodoro-cli` directory.

```bash
$ mkdir -p ~/.config/pomodoro-cli
$ cp /path/to/icon.png ~/.config/pomodoro-cli/icon.png
```

# Waybar integration

Add the following module to your waybar configuration:

![Waybar](./assets/screenshot_waybar.png)

```json
"custom/pomodoro": {
    "format": "   {}",
    "tooltip": false,
    "exec": "pomodoro-cli status --format human",
    "on-click": "pomodoro-cli toggle",
    "on-click-middle": "pomodoro-cli reset --duration 5m",
    "on-click-right": "pomodoro-cli add --duration 5m",
    "interval": 1
}
```

# Alternatives

- [i3-gnome-pomodoro](https://github.com/kantord/i3-gnome-pomodoro)
- [rust-cli-pomodoro](https://crates.io/crates/rust-cli-pomodoro)
- [pomo](https://kevinschoon.github.io/pomo/)

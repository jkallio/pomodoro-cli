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

### Download binary

- [pomodoro cli (v.1.2.1)](https://github.com/jkallio/pomodoro-cli/releases/tag/v1.2.1)

### Cargo

```bash
$ cargo install pomodoro-cli
```

# Features

- [x] Start/Stop the Timer
- [x] Query the Timer status
- [x] Add more time to a running timer.
- [x] Wait for the Timer to finish (v1.2.0)
- [x] Triggers system notification when the Timer is finished
- [x] Play alarm sound when the Timer is finished
- [x] Easy Waybar integration
- [x] Customize notification icon and alarm sound

# Usage

Options for `start`:
- `--duration` Set the duration for the timer (format: `1h 30m 15s`)
- `--notify` Triggers system notification when the timer is finished (default: disabled)
- `--silent` Do not play alarm sound when the timer is finished (default: enabled)
- `--wait` Wait for the timer to finish (default: disabled)

### Start/Stop the timer

```bash
# Start the timer with default configuration
$ pomodoro-cli start

# Start the timer with custom configuration
$ pomodoro-cli start --duration "1h 30m 15s" --silent --notify

# Stop the timer
$ pomodoro-cli stop

# Pause the Timer (calling this command again will resume the timer)
$ pomodoro-cli pause
```

### Add more time to a running timer

```bash
# Call start again while the timer is running to add more time to the timer
$ pomodoro-cli start --duration 5m
```

### Query the timer status

```bash
# Get remaining time in seconds (This is the default behavior for `status`)
$ pomodoro-cli status --format seconds

## Get remaining time in human readable format
$ pomodoro-cli status --format human

# Get the timer status in JSON format (for Waybar integration)
$ pomodoro-cli status --format json
```

# Waybar integration

![Waybar](./assets/screenshot_waybar.png)

Add the following module to your waybar configuration:

```json
"custom/pomo": {
    "format": "   {}",
    "exec": "pomodoro-cli status --format json",
    "return-type": "json",
    "on-click": "pomodoro-cli start --duration 5m --notify",
    "on-click-middle": "pomodoro-cli pause",
    "on-click-right": "pomodoro-cli stop",
    "interval": 1
},
```

### CSS styling

The module supports three different states: `running`, `paused` and `stopped`. You can customize the styling of each state by adding the following CSS rules to your Waybar configuration:

```css
#custom-pomo.finished {
  background: #8F0808;
}

#custom-pomo.running {
  background: #304D30;
}

#custom-pomo.paused {
  background: #AB730A;
}
```

###  Update Waybar module immediately

If you want to signal Waybar to update the module immediately when you can add `pkill -SIGRTMIN+10 waybar` to the `on-click` commands. For example:

```json
"custom/pomo": {
    "on-click": "pomodoro-cli start --duration 5m; pkill -SIGRTMIN+10 waybar",
    "signal": 10,
}
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

# Alternatives

- [i3-gnome-pomodoro](https://github.com/kantord/i3-gnome-pomodoro)
- [openpomodoro-cli](https://github.com/open-pomodoro/openpomodoro-cli)
- [rust-cli-pomodoro](https://crates.io/crates/rust-cli-pomodoro)
- [pomo](https://kevinschoon.github.io/pomo/)

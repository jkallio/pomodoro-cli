<div align="center">

# ![](./assets/icon.png) Pomodoro Timer Command Line Interface

Pomodoro timer is a simple timer that helps you to stay focused on your tasks.

`pomodoro-cli` is a CLI application which implements the basic functionalities of a basic Pomodoro timer. This application was designed to be easily integrated with status polling applications such as [waybar](https://github.com/Alexays/Waybar).

[![Rust](https://img.shields.io/badge/Rust-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/pomodoro-cli.svg)](https://crates.io/crates/pomodoro-cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)

</div>

# Installation

### Download binary

- [pomodoro cli (latest release)](https://github.com/jkallio/pomodoro-cli/releases/latest)

### Cargo

```bash
$ cargo install pomodoro-cli
```

# Features

- [x] Start/Stop the Timer
- [x] Query the Timer status
- [x] Create a reusable timer profiles
- [x] Add more time to a running timer.
- [x] Wait for the Timer to finish
- [x] Add custom message to the timer status
- [x] Triggers system notification when the Timer is finished
- [x] Play alarm sound when the Timer is finished
- [x] Easy Waybar integration
- [x] Customize notification icon and alarm sound

# Usage

### Start/Stop the timer

Options for `start`:
- `--duration` Set the duration for the timer (format: `1h 30m 15s`)
- `--add` Add more time to a running timer instead of starting a new timer (format: `1h 30m 15s`)
- `--message` Add a custom message to the timer status
- `--resume` Resume a paused timer (default: disabled)
- `--notify` Triggers system notification when the timer is finished (default: disabled)
- `--silent` Do not play alarm sound when the timer is finished (default: enabled)
- `--wait` Wait for the timer to finish (default: disabled)
- `--profile` Use a predefined timer profile

```bash
# Start the timer with default configuration (25 min with alarm sound)
$ pomodoro-cli start

# Start a 30 min timer wihout playing alarm sound, but triggering a system notification
$ pomodoro-cli start --duration "30m" --silent --notify

# Start a predefined timer profile
$ pomodoro-cli start --profile "work"

# Start a default Pomodoro timer profile (4 x 25m work sessions with 5m short breaks in between, and 15m long break)
$ pomodoro-cli start --profile default

# Stop the timer
$ pomodoro-cli stop
```

### Pause/Resume the timer

```bash
# Pause the Timer (calling this command again will resume the timer)
$ pomodoro-cli pause

# Resume a paused timer
$ pomodoro-cli start --resume
```

### Add more time to a running timer

```bash
# Add 10 minutes to the timer (instead of starting a new timer)
$ pomodoro-cli start -add 10m
```

### Create a reusable timer profile

Options for `new`:
- `--profile` Set the name of the profile 
- `--sequence` Set the sequence of the timer (eg: `25m 5m 25m 15m`)
- `--messages` Set the messages for each timer sequence (eg: `Work` `Short break` `Work` `Long break`)
- `--repeat` Set the number of times the sequence should be repeated (default: 1)
- `--silent` Do not play alarm sound when the timer is finished (default: disabled)
- `--notify` Triggers system notification when the timer is finished (default: disabled)
- `--alert_path` Set the path to the alarm sound (default: `~/.config/pomodoro-cli/alarm.mp3`)
- `--icon_path` Set the path to the notification icon (default: `~/.config/pomodoro-cli/icon.png`)

```bash
# Create a new timer profile
$ pomodoro-cli new --profile "work" \
--sequence 25m 5m 25m 15m \
--messages "Work" "Short break" "Work" "Long break" \
--repeat 4 \
--notify \
--alert_path "/path/to/alarm.mp3" \
--icon_path "/path/to/icon.png"
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
    "on-click": "pomodoro-cli start --add 5m --notify",
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
    "on-click": "pomodoro-cli start --add 5m; pkill -SIGRTMIN+10 waybar",
    "signal": 10,
}
```

# Customization

## Set custom alarm sound

If you want to use a custom alarm sound, you can give the path to the sound file with the `--alert_path` option. Alternatively, you can just add a `alarm.mp3` file in the `~/.config/pomodoro-cli` directory.

```bash
# Give the path to the alarm sound
$ pomodoro-cli start --alert_path "/path/to/alarm.mp3"

# Copy the alarm sound to the default location
$ mkdir -p ~/.config/pomodoro-cli
$ cp /path/to/alarm.mp3 ~/.config/pomodoro-cli/alarm.mp3
```

## Set custom notification icon 

![Waybar](./assets/screenshot_notification.png)

If you want to use a custom notification icon, you can give the path to the icon file with the `--icon_path` option. Alternatively, you can just add a `icon.png` file in the `~/.config/pomodoro-cli` directory.

```bash
# Give the path to the icon 
$ pomodoro-cli start --icon_path "/path/to/icon.png"

# Copy the icon to the default location
$ mkdir -p ~/.config/pomodoro-cli
$ cp /path/to/icon.png ~/.config/pomodoro-cli/icon.png
```

# Alternatives

- [i3-gnome-pomodoro](https://github.com/kantord/i3-gnome-pomodoro)
- [openpomodoro-cli](https://github.com/open-pomodoro/openpomodoro-cli)
- [rust-cli-pomodoro](https://crates.io/crates/rust-cli-pomodoro)
- [pomo](https://kevinschoon.github.io/pomo/)

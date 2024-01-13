# Pomodoro Timer

Pomodoro timer is a simple timer that helps you to stay focused on your tasks.

`pomodoro-cli` is a simple CLI application which implements the functionalities of a basic Pomodoro timer.

## Installation

TODO

## Features

### Configure the timer

```bash
# Reset the timer with the default values
$ pomodoro-cli reset --default

# Reset the timer with custom values
$ pomodoro-cli reset --duration 25m --notify true --silent false --wait false
```

### Start/Stop the timer

```bash
# Start the timer
$ pomodoro-cli start

# Stop the timer
$ pomodoro-cli stop

# Toggle the timer (start/stop)
$ pomodoro-cli start --toggle
```

## Query the timer status

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

### Trigger notification when the timer is finished

```bash
# Trigger notification alert when the timer is finished
pomo start --notify "Time to take a break!"
```

### Restart the timer with the previous configuration

```bash
# Start a new timer with the previous configuration
pomo restart
```

## Waybar integration

Just add the following module to your waybar configuration:

```json
"custom/pomo": {
    "format": "  {}",
    "tooltip": false,
    "exec": "pomodoro-cli status --human",
    "on-click": "pomodoro-cli start --toggle",
    "on-click-right": "pomodoro-cli reset --default",
    "on-click-right": "pomodoro-cli add 5m",
    "interval": 1
}
```


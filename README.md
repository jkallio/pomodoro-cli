# Pomodoro Timer

Pomodoro timer is a simple timer that helps you to stay focused on your tasks.

`pomodoro-cli` is a simple CLI application which implements the functionalities of a basic Pomodoro timer.

## Installation

TODO

## Features

```bash
# Start a new timer (default duration is 25 minutes)
pomo start

# Stop the active timer 
pomo stop

# Start a new timer with the same duration as the previous one
pomo restart

# Start a new timer with a custom duration
pomo start --duration 1h30m15s 

# Start a new timer with a custom task name
pomo start --task "Time to work!"

# Wait until the timer is finished
pomo start --wait

# Trigger notification alert when the timer is finished
pomo start --notify "Time to take a break!"

# Get the current timer status
pomo status
```



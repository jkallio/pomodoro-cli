# Changelog

## [1.4.1] - 2026-05-23

### Added
- `set-cycle` command to define a custom Pomodoro cycle stored in `~/.config/pomodoro-cli/cycle.json`; no arguments resets to the built-in default (4×25 min work + short/long breaks)
- `--cycle` flag for `start` to run the saved cycle with automatic phase advancement on completion
- `--repeat` flag for `start` to loop a single timer indefinitely using the previous duration, or loop an entire cycle when combined with `--cycle`

## [1.3.2] - 2026-05-19

### Fixed
- JSON output showing human-readable text field
- Waybar showing negative numbers after timer finished

## [1.3.1] - 2026-05-18

### Added
- `--watch` flag for Waybar continuous streaming support
- Integration tests for core functionality

### Changed
- Updated to Rust edition 2024
- Simplified wait mode implementation
- Cleaned up start_timer function signature
- Updated dependencies

### Fixed
- Issue where duration was silently converted to 0

## [1.2.5] - 2024-03-26

### Changed
- Use the `lock` crate instead of keyboard shortcuts to lock the screen
- Updated README documentation

## [1.2.4] - 2024-02-24

### Added
- Support for digital time format

## [1.2.3] - 2024-01-20

### Added
- ASCII animation for `--wait` flag
- Support for custom timer message

## [1.2.2] - 2024-01-20

### Added
- `--resume` flag for start command to resume paused timer
- `--add` argument for start command to add more time to a running timer

### Changed
- Prevent using conflicting command arguments

## [1.2.1] - 2024-01-19

### Fixed
- README documentation updates

## [1.2.0] - 2024-01-19

### Added
- Support for `--wait` flag to block until timer completes
- Proper error handling throughout the application

### Changed
- Don't trigger alarms and notifications on get_status when `--wait` flag is enabled
- Read help metadata directly from Cargo.toml

## [1.1.0] - 2024-01-16

### Added
- Support for JSON output format

### Fixed
- Waybar integration instructions in README

## [1.0.0] - 2024-01-15

### Added
- Documentation
- MIT License
- Support for playing alarm sound after timer finishes
- Support for adding time to running timer
- Pause and resume timer functionality
- Command line argument parsing with SubCommands

### Changed
- Changed default alarm sound

### Fixed
- Pausing and resuming timer issues
- Command line arguments now properly scoped to SubCommands

## [0.1.0] - 2024-01-13

### Added
- Initial release with basic Pomodoro timer functionality
- Start, stop, pause, and resume timer commands
- Basic status reporting

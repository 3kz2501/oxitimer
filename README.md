# oxitimer

A flexible workout/pomodoro timer CLI application written in Rust.

## Features

- **Work and break intervals** with customizable durations
- **Cycle limiting** - run a specific number of cycles or infinitely
- **Flexible time formats** - specify durations in seconds, minutes, or hours
- **Countdown** before starting the timer
- **Custom sound support** - use your own audio files (.wav/.mp3) for notifications
- **Terminal UI** with large 7-segment style timer display
- **Keyboard controls** - pause, skip, and quit
- **Audio notifications** - different sounds for work end, break end, and completion

## Installation

```bash
cargo build --release
```

The binary will be located at `./target/release/oxitimer`.

## Usage

### Basic Usage

```bash
# Positional arguments: WORK BREAK [CYCLES]
oxitimer 25 5          # 25 sec work, 5 sec break, infinite cycles
oxitimer 25 5 3        # 25 sec work, 5 sec break, 3 cycles
```

### Named Arguments

```bash
# Using named options
oxitimer --work 25 --break 5 --cycles 3
oxitimer -w 25 -b 5 -c 3
```

### Time Formats

By default, durations are in seconds. Use `--time-format` or `-t` to change:

```bash
# Minutes
oxitimer -w 25 -b 5 -t min         # 25 minutes work, 5 minutes break

# Hours
oxitimer -w 1 -b 0.25 -t hour      # 1 hour work, 15 minutes break

# Seconds (default)
oxitimer -w 1500 -b 300 -t sec     # 1500 seconds work, 300 seconds break
```

Supported format values: `sec`, `second`, `seconds`, `s`, `min`, `minute`, `minutes`, `m`, `hour`, `hours`, `h`

### Countdown

Customize the countdown before the timer starts:

```bash
# Custom countdown (always in seconds)
oxitimer 25 5 -C 10                # 10 second countdown

# Default countdown is 5 seconds
oxitimer 25 5                      # 5 second countdown

# No countdown
oxitimer 25 5 -C 0                 # Start immediately
```

## Keyboard Controls

During the timer:

- **Space** - Pause/Resume
- **Enter** - Skip to next session (plays beep sound)
- **Q** - Quit

## Custom Sounds

### Initialize Sound Directories

```bash
oxitimer init
```

This creates the following directory structure:

```
~/.config/oxitimer/sounds/
├── work_end/     # Sound when work session ends
├── break_end/    # Sound when break session ends
├── at_finish/    # Sound when all cycles complete
└── countdown/    # Sound at countdown start
```

### Adding Custom Sounds

1. Place `.wav` or `.mp3` files in the appropriate directories
2. Only one sound file per directory is used (first found)
3. If no custom sound is found, a default beep will play

## Sound Behavior

- **Work/Break completion** (natural timer end): Plays WorkEnd/BreakEnd sound
- **Manual skip** (Enter key): Plays beep sound only
- **Final cycle completion**: Plays Finish sound only (no WorkEnd sound to avoid overlap)
- **Countdown**: Plays countdown sound at start, then beeps at 3-2-1 seconds

All sounds are preloaded at startup for instant playback with no delay.

## Examples

### Pomodoro Timer (25min work, 5min break, 4 cycles)

```bash
oxitimer -w 25 -b 5 -c 4 -t min
```

### Workout Timer (30sec work, 10sec rest, 10 cycles)

```bash
oxitimer 30 10 10
```

### Study Session (50min work, 10min break, 3 cycles, no countdown)

```bash
oxitimer -w 50 -b 10 -c 3 -t min -C 0
```

### Infinite Timer (1 hour work, 15 min break)

```bash
oxitimer -w 1 -b 0.25 -t hour
```

## Display Information

The terminal UI shows:

- **Session type** - Work or Break (with color coding)
- **Timer** - Large 7-segment style display (MM:SS)
- **Status** - Running, Paused, or Completed
- **Cycle counter** - Current cycle / Total cycles (e.g., "Cycle: 2/4")
- **Controls** - Keyboard shortcuts

## Cycle Behavior

- Cycles start at 1 (not 0) for intuitive display
- A cycle consists of: Work session → Break session
- The final break is skipped - the timer finishes immediately after the last work session
- Example with 2 cycles:
  ```
  Cycle 1: Work → Break →
  Cycle 2: Work → Finished (no break)
  ```

## Architecture

The project follows a layered architecture:

- **Domain Layer** (`src/domain/`) - Core business logic
  - Session state management
  - Timer logic
  - Session types (Work/Break)

- **Infrastructure Layer** (`src/infrastructure/`) - External interfaces
  - Terminal UI rendering
  - Audio playback with rodio
  - 7-segment digit display

- **Application Layer** (`src/main.rs`) - Application entry point
  - CLI argument parsing
  - Main event loop
  - Keyboard input handling

## Dependencies

- `clap` - Command-line argument parsing
- `crossterm` - Terminal manipulation
- `tokio` - Async runtime
- `rodio` - Audio playback
- `anyhow` - Error handling

## License

MIT License

Copyright (c) 2025

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

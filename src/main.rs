mod domain;
mod infrastructure;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal,
};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time;

use domain::{SessionState, SessionType, TimerSession};
use infrastructure::{AudioPlayer, SoundEvent, TerminalUI};

#[derive(Parser)]
#[command(name = "oxitimer")]
#[command(about = "A workout/pomodoro timer with work and break intervals", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Work duration
    #[arg(short = 'w', long, value_name = "DURATION", global = true)]
    work: Option<u64>,

    /// Break duration
    #[arg(short = 'b', long, value_name = "DURATION", global = true)]
    break_duration: Option<u64>,

    /// Number of cycles to run (work + break = 1 cycle). If not specified, runs indefinitely
    #[arg(short = 'c', long, value_name = "COUNT", global = true)]
    cycles: Option<usize>,

    /// Countdown duration in seconds before starting the timer. Always in seconds regardless of time-format. Default: 5
    #[arg(short = 'C', long = "count-down", value_name = "SECONDS", default_value = "5", global = true)]
    countdown: u64,

    /// Time format: 'sec' (seconds), 'min' (minutes), 'hour' (hours). Default: sec
    #[arg(short = 't', long = "time-format", value_name = "FORMAT", default_value = "sec", global = true)]
    time_format: String,

    /// Positional arguments: WORK BREAK [CYCLES]
    #[arg(value_name = "ARGS", global = true)]
    args: Vec<u64>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize configuration directory and sound folders
    Init,
}

impl Cli {
    fn parse_args(self) -> Result<(u64, u64, Option<usize>, u64)> {
        // Parse time format
        let multiplier = match self.time_format.to_lowercase().as_str() {
            "sec" | "second" | "seconds" | "s" => 1,
            "min" | "minute" | "minutes" | "m" => 60,
            "hour" | "hours" | "h" => 3600,
            _ => return Err(anyhow::anyhow!("Invalid time format: '{}'. Use 'sec', 'min', or 'hour'", self.time_format)),
        };

        let (work, break_duration, cycles) = if !self.args.is_empty() {
            // Positional argument mode
            let work = self.args.get(0).copied()
                .ok_or_else(|| anyhow::anyhow!("Work duration is required"))?;
            let break_duration = self.args.get(1).copied()
                .ok_or_else(|| anyhow::anyhow!("Break duration is required"))?;
            let cycles = self.args.get(2).copied().map(|c| c as usize);
            (work, break_duration, cycles)
        } else {
            // Named argument mode
            let work = self.work
                .ok_or_else(|| anyhow::anyhow!("Work duration is required (use --work or positional args)"))?;
            let break_duration = self.break_duration
                .ok_or_else(|| anyhow::anyhow!("Break duration is required (use --break or positional args)"))?;
            (work, break_duration, self.cycles)
        };

        // Convert to seconds
        let work_seconds = work * multiplier;
        let break_seconds = break_duration * multiplier;

        Ok((work_seconds, break_seconds, cycles, self.countdown))
    }
}

fn initialize_config_directory() -> Result<()> {
    use std::fs;

    let config_dir = if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home).join(".config/oxitimer")
    } else {
        std::path::PathBuf::from(".oxitimer")
    };

    let sounds_dir = config_dir.join("sounds");

    // Create main directories
    println!("Creating configuration directory: {}", config_dir.display());
    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    println!("Creating sounds directory: {}", sounds_dir.display());
    fs::create_dir_all(&sounds_dir)
        .context("Failed to create sounds directory")?;

    // Create sound subdirectories
    let sound_subdirs = ["work_end", "break_end", "at_finish", "countdown"];
    for subdir in &sound_subdirs {
        let path = sounds_dir.join(subdir);
        println!("  Creating {}/", subdir);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create {} directory", subdir))?;
    }

    println!("\n✓ Configuration directory initialized successfully!");
    println!("\nSound directories created:");
    println!("  ~/.config/oxitimer/sounds/work_end/    - Sound when work session ends");
    println!("  ~/.config/oxitimer/sounds/break_end/   - Sound when break session ends");
    println!("  ~/.config/oxitimer/sounds/at_finish/   - Sound when all cycles complete");
    println!("  ~/.config/oxitimer/sounds/countdown/   - Sound at countdown start");
    println!("\nPlace .wav or .mp3 files in these directories to use custom sounds.");
    println!("If no files are found, default beep sounds will be used.");

    Ok(())
}

async fn run_countdown(countdown_seconds: u64, audio_player: &AudioPlayer) -> Result<()> {
    // Play countdown start sound if available
    // rodio's play_raw is already non-blocking, so we just call it directly
    let _ = audio_player.play_sound(SoundEvent::Countdown);

    // Clear screen and show countdown
    execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;

    for remaining in (1..=countdown_seconds).rev() {
        // Clear and display countdown
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(20, 10),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Yellow),
            crossterm::style::Print(&format!("Starting in: {} seconds", remaining)),
            crossterm::style::ResetColor
        )?;
        io::stdout().flush()?;

        // Play beep for last 3 seconds
        // rodio's play_raw is already non-blocking
        if remaining <= 3 {
            let _ = audio_player.play_beep();
        }

        time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn run_timer(work_seconds: u64, break_seconds: u64, max_cycles: Option<usize>, countdown_seconds: u64) -> Result<()> {
    // Initialize
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen)?;

    let audio_player = AudioPlayer::new()?;

    // Run countdown if specified
    if countdown_seconds > 0 {
        run_countdown(countdown_seconds, &audio_player).await?;
    }

    let mut session = TimerSession::new(work_seconds, break_seconds, max_cycles);

    let result = async {
        let mut interval = time::interval(Duration::from_millis(100));

        loop {
            // Update timer
            let session_completed = session.update();

            // Render UI
            TerminalUI::render(&session)?;

            // Handle session completion
            if session_completed {
                time::sleep(Duration::from_millis(500)).await;
                session.advance_to_next_session();

                // Check if we've transitioned to finished state
                if session.state() == SessionState::Finished {
                    // Only play Finish sound, not the end sound
                    let _ = audio_player.play_sound(SoundEvent::Finish);
                    break;
                } else {
                    // Play appropriate sound based on the previous session type
                    // Note: session_type has already been updated by advance_to_next_session
                    let sound_event = match session.session_type() {
                        SessionType::Work => SoundEvent::BreakEnd, // We just finished Break
                        SessionType::Break => SoundEvent::WorkEnd, // We just finished Work
                    };
                    let _ = audio_player.play_sound(sound_event);
                }
            }

            // Handle input (non-blocking)
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char(' ') => session.toggle_pause(),
                        KeyCode::Enter => {
                            // Play beep when manually skipping
                            let _ = audio_player.play_beep();

                            session.complete_session();
                            time::sleep(Duration::from_millis(500)).await;
                            session.advance_to_next_session();

                            // Check if finished after manual skip
                            if session.state() == SessionState::Finished {
                                let _ = audio_player.play_sound(SoundEvent::Finish);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }

            interval.tick().await;
        }

        // Show finish screen with 5 second auto-exit
        if session.is_finished() {
            TerminalUI::render(&session)?;

            let start = tokio::time::Instant::now();
            let timeout = Duration::from_secs(5);

            loop {
                // Check if timeout reached
                if start.elapsed() >= timeout {
                    break;
                }

                // Check for user input to exit early
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                        match code {
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter => break,
                            _ => {}
                        }
                    }
                }

                time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok::<(), anyhow::Error>(())
    }
    .await;

    // Cleanup
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::Init => {
                return initialize_config_directory();
            }
        }
    }

    // Regular timer execution
    let (work_seconds, break_seconds, cycles, countdown) = cli.parse_args()?;

    let cycle_info = cycles
        .map(|c| format!(" for {} cycle(s)", c))
        .unwrap_or_else(|| " (infinite)".to_string());

    println!(
        "Starting oxitimer: {}s work, {}s break{}",
        work_seconds, break_seconds, cycle_info
    );

    if countdown > 0 {
        println!("Countdown: {}s", countdown);
    }

    run_timer(work_seconds, break_seconds, cycles, countdown).await?;

    println!("Timer finished!");
    Ok(())
}

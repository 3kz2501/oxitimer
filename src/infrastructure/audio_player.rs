use anyhow::{Context, Result};
use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

/// Types of sound events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEvent {
    WorkEnd,
    BreakEnd,
    Finish,
    Countdown,
    CountdownBeep,
}

impl SoundEvent {
    fn directory_name(&self) -> &str {
        match self {
            SoundEvent::WorkEnd => "work_end",
            SoundEvent::BreakEnd => "break_end",
            SoundEvent::Finish => "at_finish",
            SoundEvent::Countdown => "countdown",
            SoundEvent::CountdownBeep => "", // Uses default beep
        }
    }

    fn all_events() -> Vec<SoundEvent> {
        vec![
            SoundEvent::WorkEnd,
            SoundEvent::BreakEnd,
            SoundEvent::Finish,
            SoundEvent::Countdown,
        ]
    }
}

/// Preloaded audio data stored as raw samples
type PreloadedSound = Arc<Vec<f32>>;

/// Audio player for notification sounds
#[derive(Clone)]
pub struct AudioPlayer {
    _stream: Arc<OutputStream>,
    stream_handle: Arc<OutputStreamHandle>,
    preloaded_sounds: Arc<HashMap<SoundEvent, PreloadedSound>>,
    sample_rate: u32,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .context("Failed to initialize audio output")?;

        // Get sound directory path
        let sound_base_path = Self::get_sound_directory();

        // Preload all sound files
        let preloaded_sounds = Self::preload_sounds(&sound_base_path)?;

        Ok(Self {
            _stream: Arc::new(stream),
            stream_handle: Arc::new(stream_handle),
            preloaded_sounds: Arc::new(preloaded_sounds),
            sample_rate: 44100,
        })
    }

    fn preload_sounds(sound_base_path: &PathBuf) -> Result<HashMap<SoundEvent, PreloadedSound>> {
        let mut sounds = HashMap::new();

        for event in SoundEvent::all_events() {
            if let Some(sound_path) = Self::find_sound_file_static(sound_base_path, event) {
                match Self::load_sound_to_memory(&sound_path) {
                    Ok(samples) => {
                        sounds.insert(event, Arc::new(samples));
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load sound for {:?}: {}", event, e);
                    }
                }
            }
        }

        Ok(sounds)
    }

    fn load_sound_to_memory(path: &PathBuf) -> Result<Vec<f32>> {
        let file = File::open(path)
            .context(format!("Failed to open sound file: {}", path.display()))?;
        let source = Decoder::new(BufReader::new(file))
            .context("Failed to decode audio file")?;

        // Convert to standard sample rate and collect all samples into memory
        let samples: Vec<f32> = source
            .convert_samples()
            .collect();

        Ok(samples)
    }

    fn get_sound_directory() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config/oxitimer/sounds")
        } else {
            PathBuf::from(".oxitimer/sounds")
        }
    }

    fn find_sound_file_static(sound_base_path: &PathBuf, event: SoundEvent) -> Option<PathBuf> {
        if matches!(event, SoundEvent::CountdownBeep) {
            return None; // Always use beep for countdown beeps
        }

        let dir = sound_base_path.join(event.directory_name());
        if !dir.exists() {
            return None;
        }

        // Look for .wav or .mp3 files
        for ext in &["wav", "mp3"] {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    pub fn play_sound(&self, event: SoundEvent) -> Result<()> {
        // Try to play preloaded sound
        if let Some(samples) = self.preloaded_sounds.get(&event) {
            let source = rodio::buffer::SamplesBuffer::new(1, self.sample_rate, samples.as_ref().clone());
            self.stream_handle.play_raw(source.convert_samples())?;
            return Ok(());
        }

        // Fallback to default beep
        self.play_beep()
    }

    pub fn play_beep(&self) -> Result<()> {
        // Generate a simple beep sound (sine wave at 800Hz for 0.5 seconds)
        let sample_rate = 44100;
        let frequency = 800.0;
        let duration = 0.5;

        let samples: Vec<f32> = (0..(sample_rate as f32 * duration) as usize)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (t * frequency * 2.0 * std::f32::consts::PI).sin() * 0.3
            })
            .collect();

        let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples);
        self.stream_handle
            .play_raw(source.convert_samples())
            .context("Failed to play sound")?;

        Ok(())
    }
}

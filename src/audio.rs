use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::constants::audio::{calculate_playback_rate, DEFAULT_VOLUME, MIN_PLAYBACK_RATE, MAX_PLAYBACK_RATE};

/// Audio engine that plays actual audio files instead of generating procedural sounds
pub struct AudioEngine {
    _stream: OutputStream,
    sink: Sink,
    enabled: bool,
    /// Playback rate that changes based on disk IOPS (higher IOPS = faster audio)
    playback_rate: f32,
}

impl AudioEngine {
    /// Creates a new audio engine with default playback rate of 1.0
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                match Sink::try_new(&stream_handle) {
                    Ok(sink) => {
                        sink.set_volume(DEFAULT_VOLUME);
                        Some(Self {
                            _stream: stream,
                            sink,
                            enabled: true,
                            playback_rate: 1.0, // Default playback rate
                        })
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    /// Updates the playback rate based on the disk IOPS (Input/Output Operations Per Second)
    /// Uses the JavaScript formula: maps IOPS [0, 16] to rate [0.5, 4.0]
    /// Higher IOPS means faster audio playback, simulating faster disk performance
    pub fn set_iops(&mut self, iops: u32) {
        self.playback_rate = calculate_playback_rate(iops);
    }
    
    /// Gets the current playback rate
    pub fn get_playback_rate(&self) -> f32 {
        self.playback_rate
    }

    /// Plays a sound file from the static/audio directory with the current playback rate
    fn play_sound_file<P: AsRef<Path>>(&self, sound_path: P) {
        if !self.enabled {
            return;
        }

        // Try to load the sound file from the static/audio directory
        let full_path = Path::new("static/audio").join(sound_path);

        // Attempt to load and play the audio file with the playback rate
        if let Ok(file) = File::open(&full_path) {
            let reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(reader) {
                // Apply playback rate to the audio source
                let source_with_rate = source.speed(self.playback_rate);
                self.sink.append(source_with_rate);
            }
        } else {
            // Try relative path if absolute fails
            let relative_path = Path::new("static").join("audio").join(&full_path);
            if let Ok(file) = File::open(&relative_path) {
                let reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(reader) {
                    // Apply playback rate to the audio source
                    let source_with_rate = source.speed(self.playback_rate);
                    self.sink.append(source_with_rate);
                }
            }
        }
    }

    /// Plays the HDD sound file which changes speed based on IOPS
    pub fn play_hdd_sound(&self) {
        self.play_sound_file("hdd.mp3");
    }

    /// Plays mouse down sound
    pub fn play_mouse_down(&self) {
        self.play_sound_file("mousedown.mp3");
    }

    /// Plays mouse up sound
    pub fn play_mouse_up(&self) {
        self.play_sound_file("mouseup.mp3");
    }

    /// Plays chimes sound for donations
    pub fn play_chimes(&self) {
        self.play_sound_file("chimes.mp3");
    }

    /// Toggles audio on/off
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if !self.enabled {
            self.sink.stop();
        }
    }

    /// Checks if audio is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    // For compatibility with existing code - these functions map to the new sound files
    pub fn play_seek(&self) {
        // Use the hdd sound for seek operations
        self.play_sound_file("hdd.mp3");
    }

    pub fn play_read(&self) {
        // Use the hdd sound for read operations
        self.play_sound_file("hdd.mp3");
    }

    pub fn play_write(&self) {
        // Use the hdd sound for write operations
        self.play_sound_file("hdd.mp3");
    }
}
use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::Cursor;

// Embedded resources module for audio files
mod resources {
    use std::io::Cursor;

    /// Embedded HDD sound file (hdd.mp3)
    pub const HDD_SOUND: &'static [u8] = include_bytes!("../static/audio/hdd.mp3");

    /// Embedded mouse down sound file (mousedown.mp3)
    pub const MOUSE_DOWN_SOUND: &'static [u8] = include_bytes!("../static/audio/mousedown.mp3");

    /// Embedded mouse up sound file (mouseup.mp3)
    pub const MOUSE_UP_SOUND: &'static [u8] = include_bytes!("../static/audio/mouseup.mp3");

    /// Embedded chimes sound file (chimes.mp3)
    pub const CHIMES_SOUND: &'static [u8] = include_bytes!("../static/audio/chimes.mp3");

    /// Embedded loop sound file (loop.mp3)
    pub const LOOP_SOUND: &'static [u8] = include_bytes!("../static/audio/loop.mp3");

    /// A structure to hold all embedded audio resources
    pub struct EmbeddedAudioResources;

    impl EmbeddedAudioResources {
        /// Returns a cursor for the HDD sound file
        pub fn hdd_sound() -> Cursor<&'static [u8]> {
            Cursor::new(HDD_SOUND)
        }

        /// Returns a cursor for the mouse down sound file
        pub fn mouse_down_sound() -> Cursor<&'static [u8]> {
            Cursor::new(MOUSE_DOWN_SOUND)
        }

        /// Returns a cursor for the mouse up sound file
        pub fn mouse_up_sound() -> Cursor<&'static [u8]> {
            Cursor::new(MOUSE_UP_SOUND)
        }

        /// Returns a cursor for the chimes sound file
        pub fn chimes_sound() -> Cursor<&'static [u8]> {
            Cursor::new(CHIMES_SOUND)
        }

        /// Returns a cursor for the loop sound file
        pub fn loop_sound() -> Cursor<&'static [u8]> {
            Cursor::new(LOOP_SOUND)
        }
    }
}

use resources::EmbeddedAudioResources;

/// Audio engine that plays embedded audio files instead of generating procedural sounds
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
                        sink.set_volume(0.5);
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
    /// Higher IOPS means faster audio playback, simulating faster disk performance
    pub fn set_iops(&mut self, iops: u32) {
        // Calculate playback rate based on IOPS following the JavaScript formula: 1000 / iops
        // Using a minimum of 0.1 and maximum of 4.0 to avoid extreme values
        let rate = (1000.0 / (iops as f32)).max(0.1).min(4.0);
        self.playback_rate = rate;
    }
    
    /// Plays an embedded sound from memory with the current playback rate
    fn play_embedded_sound(&self, sound_data: Cursor<&'static [u8]>) {
        if !self.enabled {
            return;
        }

        // Create a decoder from the embedded sound data
        if let Ok(source) = Decoder::new(sound_data) {
            // Apply playback rate to the audio source
            let source_with_rate = source.speed(self.playback_rate);
            self.sink.append(source_with_rate);
        }
    }

    /// Plays the HDD sound file which changes speed based on IOPS
    pub fn play_hdd_sound(&self) {
        self.play_embedded_sound(EmbeddedAudioResources::hdd_sound());
    }

    /// Plays mouse down sound
    pub fn play_mouse_down(&self) {
        self.play_embedded_sound(EmbeddedAudioResources::mouse_down_sound());
    }

    /// Plays mouse up sound
    pub fn play_mouse_up(&self) {
        self.play_embedded_sound(EmbeddedAudioResources::mouse_up_sound());
    }

    /// Plays chimes sound for donations
    pub fn play_chimes(&self) {
        self.play_embedded_sound(EmbeddedAudioResources::chimes_sound());
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
    
    /// Plays a looping background sound (ambient drive noise)
    /// This creates continuous background ambiance during defragmentation
    pub fn play_loop_sound(&self) {
        self.play_embedded_sound(EmbeddedAudioResources::loop_sound());
    }
    
    /// Stops all currently playing sounds
    pub fn stop_all(&self) {
        self.sink.stop();
    }

    // For compatibility with existing code - these functions map to the new sound files
    pub fn play_seek(&self) {
        // Use the hdd sound for seek operations
        self.play_embedded_sound(EmbeddedAudioResources::hdd_sound());
    }

    pub fn play_read(&self) {
        // Use the hdd sound for read operations
        self.play_embedded_sound(EmbeddedAudioResources::hdd_sound());
    }

    pub fn play_write(&self) {
        // Use the hdd sound for write operations
        self.play_embedded_sound(EmbeddedAudioResources::hdd_sound());
    }
}
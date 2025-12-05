//! Embedded resources for the defragmentation simulator
//! This module includes all audio files directly in the binary using include_bytes!

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
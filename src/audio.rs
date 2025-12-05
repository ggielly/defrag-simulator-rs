use rodio::{OutputStream, Source, Sink};
use std::time::Duration;

// -- Audio Engine -------------------------------------------------------------

/// Générateur de son HDD procédural
pub struct HddSoundGenerator {
    sample_rate: u32,
    phase: f32,
    sound_type: HddSoundType,
    click_countdown: u32,
    rng_state: u64,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HddSoundType {
    Seek,      // Bruit de déplacement de tête (clics rapides)
    Read,      // Grattement de lecture
    Write,     // Grattement d'écriture (légèrement différent)
    Idle,      // Ronronnement de fond
}

impl HddSoundGenerator {
    pub fn new(sound_type: HddSoundType) -> Self {
        Self {
            sample_rate: 44100,
            phase: 0.0,
            sound_type,
            click_countdown: 0,
            rng_state: 12345,
        }
    }

    // Générateur de bruit pseudo-aléatoire simple (xorshift)
    fn noise(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state as f32 / u64::MAX as f32) * 2.0 - 1.0
    }

    fn generate_sample(&mut self) -> f32 {
        match self.sound_type {
            HddSoundType::Seek => self.generate_seek_sound(),
            HddSoundType::Read => self.generate_read_sound(),
            HddSoundType::Write => self.generate_write_sound(),
            HddSoundType::Idle => self.generate_idle_sound(),
        }
    }

    fn generate_seek_sound(&mut self) -> f32 {
        // Son de seek: clics mécaniques rapides
        self.phase += 1.0;

        if self.click_countdown == 0 {
            // Nouveau clic toutes les 50-150 samples
            self.click_countdown = 50 + (self.noise().abs() * 100.0) as u32;
            return 0.8 * (if self.noise() > 0.0 { 1.0 } else { -1.0 });
        }

        self.click_countdown = self.click_countdown.saturating_sub(1);

        // Bruit de fond mécanique
        let mechanical = (self.phase * 0.01).sin() * 0.1;
        let noise = self.noise() * 0.05;

        (mechanical + noise) * 0.5
    }

    fn generate_read_sound(&mut self) -> f32 {
        // Son de lecture: grattement régulier + bruit haute fréquence
        self.phase += 1.0;

        // Ton de base (moteur)
        let motor = (self.phase * 0.002 * std::f32::consts::TAU).sin() * 0.15;

        // Grattement (bruit filtré)
        let scratch = self.noise() * 0.2;

        // Modulation pour effet de "tête qui lit"
        let modulation = ((self.phase * 0.0001).sin() + 1.0) * 0.5;

        (motor + scratch * modulation) * 0.4
    }

    fn generate_write_sound(&mut self) -> f32 {
        // Son d'écriture: similaire à lecture mais plus "intense"
        self.phase += 1.0;

        let motor = (self.phase * 0.0025 * std::f32::consts::TAU).sin() * 0.2;
        let scratch = self.noise() * 0.25;
        let click = if (self.phase as u32) % 200 < 10 { 0.3 } else { 0.0 };

        (motor + scratch + click) * 0.4
    }

    fn generate_idle_sound(&mut self) -> f32 {
        // Son de repos: ronronnement léger du moteur
        self.phase += 1.0;

        let motor = (self.phase * 0.001 * std::f32::consts::TAU).sin() * 0.05;
        let noise = self.noise() * 0.02;

        (motor + noise) * 0.2
    }
}

impl Iterator for HddSoundGenerator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generate_sample())
    }
}

impl Source for HddSoundGenerator {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Infini
    }
}

/// Gestionnaire audio pour le simulateur
pub struct AudioEngine {
    _stream: OutputStream,
    sink: Sink,
    enabled: bool,
}

impl AudioEngine {
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
                        })
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    pub fn play_sound(&self, sound_type: HddSoundType, duration_ms: u64) {
        if !self.enabled {
            return;
        }

        let generator = HddSoundGenerator::new(sound_type);
        let source = generator.take_duration(Duration::from_millis(duration_ms));
        self.sink.append(source);
    }

    pub fn play_seek(&self) {
        self.play_sound(HddSoundType::Seek, 50);
    }

    pub fn play_read(&self) {
        self.play_sound(HddSoundType::Read, 80);
    }

    pub fn play_write(&self) {
        self.play_sound(HddSoundType::Write, 80);
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if !self.enabled {
            self.sink.stop();
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
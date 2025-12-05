use std::{
    io::Result,
    sync::mpsc,
    time::{Duration, Instant},
};
use crate::models::{ClusterState, DefragPhase, DefragStats};
use crate::audio::AudioEngine;
use rand::prelude::SliceRandom;

// -- CLI Arguments ------------------------------------------------------------

#[derive(clap::Parser)]
#[command(name = "defrag", version = "0.1.0", about = "MS-DOS Defragmenter Simulation")]
pub struct Args {
    /// Animation speed: fast, normal, or slow
    #[arg(long, default_value = "normal")]
    pub speed: String,

    /// Grid size in format WxH (e.g., 85x20)
    #[arg(long, default_value = "78x16")]
    pub size: String,

    /// Initial disk fill percentage
    #[arg(long, default_value_t = 0.65)]
    pub fill: f32,

    /// Enable HDD sounds
    #[arg(long, short = 's', default_value_t = false)]
    pub sound: bool,
}

// -- Disk Drive Types ----------------------------------------------------------

/// Represents different types of disk drives with different IOPS (Input/Output Operations Per Second)
/// Based on real historical performance characteristics of different disk types
#[derive(Debug, Clone)]
pub struct DiskDrive {
    pub letter: char,
    pub capacity_mb: u32,
    pub cluster_count: u32,
    pub iops: u32,  // IOPS (Input/Output Operations Per Second) - affects audio playback speed
    pub name: String,
}

impl DiskDrive {
    /// Creates a new disk drive instance
    fn new(letter: char, capacity_mb: u32, cluster_count: u32, iops: u32, name: &str) -> Self {
        Self {
            letter,
            capacity_mb,
            cluster_count,
            iops,
            name: name.to_string(),
        }
    }
}

/// Collection of available disk drives for the simulation
pub struct DiskDriveCollection {
    drives: Vec<DiskDrive>,
}

impl DiskDriveCollection {
    /// Creates the default collection of disk drives
    pub fn new() -> Self {
        Self {
            drives: vec![
                // Based on the JavaScript implementation values
                DiskDrive::new('C', 2048, 4096, 2, "Hard Disk (2GB, 2 IOPS)"),
                DiskDrive::new('D', 1024, 2048, 3, "Hard Disk (1GB, 3 IOPS)"),
                DiskDrive::new('E', 512, 1024, 1, "Floppy Disk (512MB, 1 IOPS)"),
                DiskDrive::new('F', 2048, 4096, 8, "SSHD (2GB, 8 IOPS)"), // Faster modern hybrid drive
            ],
        }
    }

    /// Gets a reference to all available drives
    pub fn get_all(&self) -> &[DiskDrive] {
        &self.drives
    }

    /// Gets a disk drive by its letter
    pub fn get_by_letter(&self, letter: char) -> Option<&DiskDrive> {
        self.drives.iter().find(|drive| drive.letter == letter)
    }

    /// Gets drive by index
    pub fn get_by_index(&self, index: usize) -> Option<&DiskDrive> {
        self.drives.get(index)
    }

    /// Gets the default drive (first one)
    pub fn get_default(&self) -> &DiskDrive {
        &self.drives[0]  // Default to drive C
    }
}

// -- Application State --------------------------------------------------------

pub struct App {
    pub running: bool,
    pub tick_rate: Duration,
    pub width: usize,
    pub height: usize,
    pub clusters: Vec<ClusterState>,
    pub stats: DefragStats,
    pub phase: DefragPhase,
    // Animation state
    pub animation_step: u64,
    pub read_pos: Option<usize>,
    pub write_pos: Option<usize>,
    // Menu state
    pub menu_open: bool,
    pub selected_menu: usize,
    pub selected_item: usize,
    // Dialog state
    pub show_about_box: bool,
    // Audio - now includes IOPS-based playback rate
    pub audio: Option<AudioEngine>,
    // Disk drive information
    pub current_drive: DiskDrive,
    pub drive_collection: DiskDriveCollection,
}

impl App {
    pub fn new(width: usize, height: usize, fill_percent: f32, enable_sound: bool) -> Self {
        let total_clusters = width * height;
        let mut rng = rand::thread_rng();

        // Calculer le nombre de clusters à défragmenter (comme dans PHP)
        let num_pending = (total_clusters as f32 * fill_percent) as usize;
        let num_bad = (total_clusters as f32 * 0.02) as usize;

        // Créer le disque avec des secteurs Pending (à défragmenter)
        let mut clusters: Vec<ClusterState> = Vec::with_capacity(total_clusters);

        // Ajouter les secteurs Pending (fichiers fragmentés à déplacer)
        for _ in 0..(num_pending.saturating_sub(2)) {
            clusters.push(ClusterState::Pending);
        }

        // Ajouter les blocs Writing et Reading initiaux
        clusters.push(ClusterState::Writing);
        clusters.push(ClusterState::Reading);

        // Compléter avec des secteurs Unused (espace libre)
        while clusters.len() < total_clusters - num_bad {
            clusters.push(ClusterState::Unused);
        }

        // Mélanger tout le disque pour simuler la fragmentation
        clusters.shuffle(&mut rng);

        // Ajouter les blocs Bad à des positions aléatoires
        let mut bad_positions: Vec<usize> = (0..clusters.len()).collect();
        bad_positions.shuffle(&mut rng);
        for &pos in bad_positions.iter().take(num_bad) {
            clusters.insert(pos.min(clusters.len()), ClusterState::Bad);
        }

        // Tronquer si nécessaire
        clusters.truncate(total_clusters);

        // Mettre un bloc Unmovable au début (comme le boot sector)
        if !clusters.is_empty() {
            clusters[0] = ClusterState::Unmovable;
        }

        let total_to_defrag = clusters.iter().filter(|&&c| c == ClusterState::Pending).count() + 2; // +2 pour Reading/Writing initiaux

        // Initialize the drive collection
        let drive_collection = DiskDriveCollection::new();
        let current_drive = drive_collection.get_default().clone();

        Self {
            running: true,
            tick_rate: Duration::from_millis(80), // Légèrement plus rapide
            width,
            height,
            clusters,
            stats: DefragStats {
                total_to_defrag,
                clusters_defragged: 0,
                start_time: Instant::now(),
            },
            phase: DefragPhase::Initializing,
            animation_step: 0,
            read_pos: None,
            write_pos: None,
            // Menu state
            menu_open: false,
            selected_menu: 0,
            selected_item: 0,
            // Dialog state
            show_about_box: false,
            // Audio engine
            audio: if enable_sound {
                let mut audio = AudioEngine::new();
                if let Some(ref mut audio_engine) = audio {
                    audio_engine.set_iops(current_drive.iops);
                }
                audio
            } else {
                None
            },
            // Disk drive information
            current_drive,
            drive_collection,
        }
    }

    pub fn run(&mut self, term: &mut crate::ui::TuiWrapper, rx: mpsc::Receiver<()>) -> Result<()> {
        use crossterm::{
            event::{self, Event, KeyCode, KeyEventKind},
        };

        let mut last_tick = Instant::now();
        while self.running {
            term.draw(|frame| crate::ui::render_app(&self, frame))?;

            // Handle Ctrl+C
            if rx.try_recv().is_ok() {
                self.running = false;
            }

            // Handle other keyboard events
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // Si About Box est ouverte, seules certaines touches la ferment
                        if self.show_about_box {
                            match key.code {
                                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                                    self.show_about_box = false;
                                }
                                _ => {}
                            }
                            continue;
                        }

                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                if self.menu_open {
                                    self.menu_open = false;
                                } else {
                                    self.running = false;
                                }
                            }
                            KeyCode::F(1) => {
                                // F1 = Help -> Afficher About
                                self.show_about_box = true;
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                // Toggle sound
                                if let Some(ref mut audio) = self.audio {
                                    audio.toggle();
                                } else {
                                    // Activer le son si pas encore initialisé
                                    self.audio = AudioEngine::new();
                                }
                            }
                            KeyCode::F(10) | KeyCode::Tab => {
                                // Ouvrir/fermer le menu
                                self.menu_open = !self.menu_open;
                                if self.menu_open {
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Left => {
                                if self.menu_open {
                                    self.selected_menu = if self.selected_menu == 0 { 4 } else { self.selected_menu - 1 };
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Right => {
                                if self.menu_open {
                                    self.selected_menu = (self.selected_menu + 1) % 5;
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Up => {
                                if self.menu_open {
                                    let max_items = crate::ui::get_menu_items(self.selected_menu).len();
                                    self.selected_item = if self.selected_item == 0 { max_items.saturating_sub(1) } else { self.selected_item - 1 };
                                }
                            }
                            KeyCode::Down => {
                                if self.menu_open {
                                    let max_items = crate::ui::get_menu_items(self.selected_menu).len();
                                    self.selected_item = (self.selected_item + 1) % max_items;
                                }
                            }
                            KeyCode::Enter => {
                                if self.menu_open {
                                    self.handle_menu_action();
                                    self.menu_open = false;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Update app state on each tick
            if last_tick.elapsed() >= self.tick_rate {
                self.update();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    pub fn update(&mut self) {
        self.animation_step += 1;

        match self.phase {
            DefragPhase::Initializing => {
                // Wait for a bit before starting analysis
                if self.animation_step > 20 { // 2 seconds
                    self.phase = DefragPhase::Analyzing;
                    self.animation_step = 0;
                }
            }
            DefragPhase::Analyzing => {
                // Simulate analysis by showing a sweeping "Reading" block
                let total_clusters = self.width * self.height;
                let scan_pos = (self.animation_step as usize * 5).min(total_clusters-1);
                self.read_pos = Some(scan_pos);

                // Jouer un son de seek pendant l'analyse
                if self.animation_step % 3 == 0 {
                    if let Some(ref audio) = self.audio {
                        audio.play_seek();
                    }
                }

                if self.animation_step > (total_clusters as u64 / 5) + 10 {
                    self.read_pos = None;
                    self.phase = DefragPhase::Defragmenting;
                    self.animation_step = 0;
                }
            }
            DefragPhase::Defragmenting => {
                let mut rng = rand::thread_rng();

                // Trouver et effacer le bloc Reading actuel (le mettre en Unused)
                if let Some(reading_idx) = self.clusters.iter().position(|&c| c == ClusterState::Reading) {
                    self.clusters[reading_idx] = ClusterState::Unused;
                    // Son de lecture
                    if let Some(ref audio) = self.audio {
                        audio.play_read();
                    }
                }

                // Trouver et convertir le bloc Writing en Used (défragmenté)
                if let Some(writing_idx) = self.clusters.iter().position(|&c| c == ClusterState::Writing) {
                    self.clusters[writing_idx] = ClusterState::Used;
                    self.stats.clusters_defragged += 1;
                    // Son d'écriture
                    if let Some(ref audio) = self.audio {
                        audio.play_write();
                    }
                }

                // Chercher un bloc Pending (à défragmenter) - choix ALÉATOIRE comme dans PHP
                let pending_indices: Vec<usize> = self.clusters
                    .iter()
                    .enumerate()
                    .filter(|&(_, c)| *c == ClusterState::Pending)
                    .map(|(i, _)| i)
                    .collect();

                if let Some(&pending_idx) = pending_indices.choose(&mut rng) {
                    // Marquer ce bloc comme Reading
                    self.clusters[pending_idx] = ClusterState::Reading;
                    self.read_pos = Some(pending_idx);

                    // Son de seek quand on change de position
                    if let Some(ref audio) = self.audio {
                        audio.play_seek();
                    }

                    // Trouver le premier bloc Unused et le marquer comme Writing
                    if let Some(unused_idx) = self.clusters.iter().position(|&c| c == ClusterState::Unused) {
                        self.clusters[unused_idx] = ClusterState::Writing;
                        self.write_pos = Some(unused_idx);
                    }
                } else {
                    // Plus de blocs Pending, défragmentation terminée
                    self.read_pos = None;
                    self.write_pos = None;
                    self.phase = DefragPhase::Finished;
                }
            }
            DefragPhase::Finished => {
                // Wait for a bit then quit
                if self.animation_step > 50 { // 5 seconds
                    self.running = false;
                }
            }
        }
    }

    fn handle_menu_action(&mut self) {
        match (self.selected_menu, self.selected_item) {
            (0, 0) => {
                // Begin optimization - restart defrag
                if self.phase == DefragPhase::Finished {
                    self.phase = DefragPhase::Defragmenting;
                    self.animation_step = 0;
                }
            }
            (0, 4) => {
                // Exit
                self.running = false;
            }
            (1, 0) => {
                // Analyze drive
                if self.phase != DefragPhase::Analyzing {
                    self.phase = DefragPhase::Analyzing;
                    self.animation_step = 0;
                }
            }
            (4, 0) | (4, 1) => {
                // Help -> Contents ou About
                self.show_about_box = true;
            }
            _ => {}
        }
    }
}

pub fn parse_size(size_str: &str) -> Result<(usize, usize)> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Size must be in format WxH"));
    }
    let width: usize = parts[0].parse().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid width"))?;
    let height: usize = parts[1].parse().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid height"))?;
    Ok((width, height))
}
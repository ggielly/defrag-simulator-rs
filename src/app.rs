use std::{
    io::Result,
    sync::mpsc,
    time::{Duration, Instant},
};
use crate::models::{ClusterState, DefragPhase, DefragStats};
use crate::audio::AudioEngine;
use crate::constants::{disk, audio as audio_const, animation, ui as ui_const, defrag_type::DefragStyle};
use rand::prelude::{SliceRandom, Rng};

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
    
    /// Select disk drive (C, D, E, or F)
    #[arg(long, short = 'd', default_value = "C")]
    pub drive: char,
    
    /// UI style: msdos, win95, or win98
    #[arg(long, short = 'u', default_value = "msdos")]
    pub ui: String,
}

impl Args {
    /// Parse the UI style from the command line argument
    pub fn get_ui_style(&self) -> DefragStyle {
        match self.ui.to_lowercase().as_str() {
            "win98" | "windows98" | "98" => DefragStyle::Windows98,
            "win95" | "windows95" | "95" => DefragStyle::Windows95,
            _ => DefragStyle::MsDos,
        }
    }
}

// -- Disk Drive Types ----------------------------------------------------------

/// Represents different types of disk drives with different IOPS (Input/Output Operations Per Second)
/// Based on real historical performance characteristics of different disk types
#[derive(Debug, Clone)]
pub struct DiskDrive {
    pub config: disk::DriveConfig,
    pub name: String,
}

impl DiskDrive {
    /// Creates a new disk drive instance from a DriveConfig
    pub fn from_config(config: disk::DriveConfig) -> Self {
        let name = match config.letter {
            'C' => "Hard Disk (2GB, 2 IOPS)",
            'D' => "Hard Disk (1GB, 3 IOPS)",
            'E' => "Floppy Disk (512MB, 1 IOPS)",
            'F' => "SSHD (2GB, 8 IOPS)",
            _ => "Unknown Drive",
        };
        Self {
            config,
            name: name.to_string(),
        }
    }
    
    /// Gets the IOPS value for this drive
    pub fn iops(&self) -> u32 {
        self.config.iops
    }
    
    /// Gets the drive letter
    pub fn letter(&self) -> char {
        self.config.letter
    }
    
    /// Gets the calculated playback rate for audio based on IOPS
    pub fn audio_playback_rate(&self) -> f32 {
        audio_const::calculate_playback_rate(self.config.iops)
    }
}

/// Collection of available disk drives for the simulation
pub struct DiskDriveCollection {
    drives: Vec<DiskDrive>,
}

impl DiskDriveCollection {
    /// Creates the default collection of disk drives from constants
    pub fn new() -> Self {
        Self {
            drives: disk::ALL_DRIVES
                .iter()
                .map(|&config| DiskDrive::from_config(config))
                .collect(),
        }
    }

    /// Gets a reference to all available drives
    pub fn get_all(&self) -> &[DiskDrive] {
        &self.drives
    }

    /// Gets a disk drive by its letter
    pub fn get_by_letter(&self, letter: char) -> Option<&DiskDrive> {
        self.drives.iter().find(|drive| drive.letter() == letter)
    }

    /// Gets drive by index
    pub fn get_by_index(&self, index: usize) -> Option<&DiskDrive> {
        self.drives.get(index)
    }

    /// Gets the default drive (Drive C)
    pub fn get_default(&self) -> &DiskDrive {
        &self.drives[0]
    }
}

// -- File Simulation --------------------------------------------------------

/// Represents a logical file with multiple clusters
/// This allows simulating files of different sizes during defragmentation
#[derive(Debug, Clone)]
pub struct FileFragment {
    /// The cluster indices that belong to this file
    pub clusters: Vec<usize>,
    /// The size of the file in clusters
    pub size: usize,
    /// Whether this file is fragmented (clusters not contiguous)
    pub is_fragmented: bool,
}

impl FileFragment {
    /// Create a new file fragment
    pub fn new(clusters: Vec<usize>) -> Self {
        let size = clusters.len();
        let is_fragmented = Self::check_fragmentation(&clusters);
        Self { clusters, size, is_fragmented }
    }
    
    /// Check if clusters are contiguous (not fragmented)
    fn check_fragmentation(clusters: &[usize]) -> bool {
        if clusters.len() <= 1 {
            return false;
        }
        for window in clusters.windows(2) {
            if window[1] != window[0] + 1 {
                return true; // Not contiguous = fragmented
            }
        }
        false
    }
    
    /// Get the first cluster of this file
    pub fn first_cluster(&self) -> Option<usize> {
        self.clusters.first().copied()
    }
    
    /// Get the last cluster of this file
    pub fn last_cluster(&self) -> Option<usize> {
        self.clusters.last().copied()
    }
}

/// Represents the state of a file during defragmentation
#[derive(Debug, Clone)]
pub enum FileDefragPhase {
    /// The file is being read from its fragmented location
    Reading { progress: usize },
    /// The file is being written to its new contiguous location
    Writing { progress: usize },
    /// The file has been fully defragmented
    Completed,
}

// -- Application State --------------------------------------------------------

/// Cache for tracking free space regions (optimization)
#[derive(Debug, Clone)]
pub struct FreeSpaceCache {
    /// List of (start_index, length) for contiguous free regions
    regions: Vec<(usize, usize)>,
    /// Whether the cache needs rebuilding
    dirty: bool,
}

impl FreeSpaceCache {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            dirty: true,
        }
    }
    
    /// Mark cache as needing rebuild
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }
    
    /// Rebuild the cache from cluster state
    pub fn rebuild(&mut self, clusters: &[ClusterState]) {
        self.regions.clear();
        let mut start: Option<usize> = None;
        let mut length = 0;
        
        for (i, &cluster) in clusters.iter().enumerate() {
            if cluster == ClusterState::Unused {
                if start.is_none() {
                    start = Some(i);
                }
                length += 1;
            } else if let Some(s) = start {
                self.regions.push((s, length));
                start = None;
                length = 0;
            }
        }
        
        // Don't forget the last region
        if let Some(s) = start {
            self.regions.push((s, length));
        }
        
        // Sort by size (largest first) for better allocation
        self.regions.sort_by(|a, b| b.1.cmp(&a.1));
        self.dirty = false;
    }
    
    /// Find a region with at least `size` contiguous clusters
    pub fn find_region(&self, size: usize) -> Option<usize> {
        self.regions.iter()
            .find(|(_, len)| *len >= size)
            .map(|(start, _)| *start)
    }
}

pub struct App {
    pub running: bool,
    pub paused: bool,  // NEW: Pause state
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
    // File defragmentation state
    pub current_file_read_progress: Option<FileDefragPhase>,
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
    // UI style (MS-DOS, Win95, Win98)
    pub ui_style: DefragStyle,
    // Performance optimization: cache of free space regions
    free_space_cache: FreeSpaceCache,
    // Demo mode: auto-restart when finished
    pub demo_mode: bool,
    // Pending clusters index cache (optimization)
    pending_indices_cache: Vec<usize>,
    pending_cache_dirty: bool,
}

impl App {
    pub fn new(width: usize, height: usize, fill_percent: f32, enable_sound: bool, drive_letter: char, ui_style: DefragStyle) -> Self {
        let total_clusters = width * height;
        let mut rng = rand::thread_rng();

        // Calculer le nombre de clusters à défragmenter (comme dans PHP)
        let num_pending = (total_clusters as f32 * fill_percent) as usize;
        let num_bad = (total_clusters as f32 * ui_const::BAD_BLOCK_PERCENT) as usize;

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

        // Initialize the drive collection and select the requested drive
        let drive_collection = DiskDriveCollection::new();
        let current_drive = drive_collection
            .get_by_letter(drive_letter.to_ascii_uppercase())
            .unwrap_or_else(|| drive_collection.get_default())
            .clone();

        Self {
            running: true,
            paused: false,
            tick_rate: Duration::from_millis(animation::DEFAULT_TICK_RATE_MS),
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
            current_file_read_progress: None,
            // Menu state
            menu_open: false,
            selected_menu: 0,
            selected_item: 0,
            // Dialog state
            show_about_box: false,
            // Audio engine with IOPS-based playback rate
            audio: if enable_sound {
                let mut audio = AudioEngine::new();
                if let Some(ref mut audio_engine) = audio {
                    audio_engine.set_iops(current_drive.iops());
                }
                audio
            } else {
                None
            },
            // Disk drive information
            current_drive,
            drive_collection,
            // UI style
            ui_style,
            // Performance caches
            free_space_cache: FreeSpaceCache::new(),
            demo_mode: false,
            pending_indices_cache: Vec::new(),
            pending_cache_dirty: true,
        }
    }
    
    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.phase == DefragPhase::Defragmenting || self.phase == DefragPhase::Analyzing {
            self.paused = !self.paused;
            // Stop audio when paused
            if self.paused {
                if let Some(ref audio) = self.audio {
                    audio.stop_all();
                }
            }
        }
    }
    
    /// Toggle demo mode (auto-restart)
    pub fn toggle_demo_mode(&mut self) {
        self.demo_mode = !self.demo_mode;
    }
    
    /// Restart defragmentation from the beginning
    pub fn restart(&mut self) {
        let mut rng = rand::thread_rng();
        let total_clusters = self.width * self.height;
        let fill_percent = ui_const::DEFAULT_FILL_PERCENT;
        
        // Recréer les clusters
        let num_pending = (total_clusters as f32 * fill_percent) as usize;
        let num_bad = (total_clusters as f32 * ui_const::BAD_BLOCK_PERCENT) as usize;
        
        self.clusters.clear();
        for _ in 0..(num_pending.saturating_sub(2)) {
            self.clusters.push(ClusterState::Pending);
        }
        self.clusters.push(ClusterState::Writing);
        self.clusters.push(ClusterState::Reading);
        while self.clusters.len() < total_clusters - num_bad {
            self.clusters.push(ClusterState::Unused);
        }
        self.clusters.shuffle(&mut rng);
        
        let mut bad_positions: Vec<usize> = (0..self.clusters.len()).collect();
        bad_positions.shuffle(&mut rng);
        for &pos in bad_positions.iter().take(num_bad) {
            self.clusters.insert(pos.min(self.clusters.len()), ClusterState::Bad);
        }
        self.clusters.truncate(total_clusters);
        if !self.clusters.is_empty() {
            self.clusters[0] = ClusterState::Unmovable;
        }
        
        // Reset stats
        let total_to_defrag = self.clusters.iter().filter(|&&c| c == ClusterState::Pending).count() + 2;
        self.stats = DefragStats {
            total_to_defrag,
            clusters_defragged: 0,
            start_time: Instant::now(),
        };
        
        // Reset state
        self.phase = DefragPhase::Initializing;
        self.animation_step = 0;
        self.read_pos = None;
        self.write_pos = None;
        self.current_file_read_progress = None;
        self.paused = false;
        
        // Invalidate caches
        self.free_space_cache.invalidate();
        self.pending_cache_dirty = true;
    }
    
    /// Calculate estimated time remaining
    pub fn estimated_time_remaining(&self) -> Option<Duration> {
        if self.stats.clusters_defragged == 0 || self.phase != DefragPhase::Defragmenting {
            return None;
        }
        
        let elapsed = self.stats.start_time.elapsed();
        let remaining = self.stats.total_to_defrag.saturating_sub(self.stats.clusters_defragged);
        
        if remaining == 0 {
            return Some(Duration::ZERO);
        }
        
        // Calculate rate (clusters per second)
        let rate = self.stats.clusters_defragged as f64 / elapsed.as_secs_f64();
        if rate <= 0.0 {
            return None;
        }
        
        let remaining_secs = remaining as f64 / rate;
        Some(Duration::from_secs_f64(remaining_secs))
    }
    
    /// Get progress percentage
    pub fn progress_percent(&self) -> f32 {
        if self.stats.total_to_defrag == 0 {
            return 100.0;
        }
        (self.stats.clusters_defragged as f32 / self.stats.total_to_defrag as f32) * 100.0
    }

    pub fn run(&mut self, term: &mut crate::ui::TuiWrapper, rx: mpsc::Receiver<()>) -> Result<()> {
        use crossterm::{
            event::{self, Event, KeyCode, KeyEventKind},
        };

        let mut last_tick = Instant::now();
        while self.running {
            // Render based on UI style
            term.draw(|frame| {
                match self.ui_style {
                    DefragStyle::Windows98 => crate::win98::render_win98_app(&self, frame),
                    DefragStyle::Windows95 => crate::win98::render_win98_app(&self, frame), // TODO: implement Win95
                    DefragStyle::MsDos => crate::ui::render_app(&self, frame),
                }
            })?;

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
                            // NEW: Pause with Space or P
                            KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => {
                                if !self.menu_open {
                                    self.toggle_pause();
                                }
                            }
                            // NEW: Restart with R
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                if !self.menu_open {
                                    self.restart();
                                }
                            }
                            // NEW: Demo mode with D
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                if !self.menu_open {
                                    self.toggle_demo_mode();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Update app state on each tick (only if not paused)
            if last_tick.elapsed() >= self.tick_rate && !self.paused {
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

                // Calculate the speed factor based on the drive's IOPS
                // Higher IOPS means faster operations (lower delay between operations)
                let speed_factor = 1.0 / (self.current_drive.iops() as f64).max(1.0);
                
                // Dynamically adjust tick rate based on drive speed
                // Faster drives (higher IOPS) = shorter tick rate = faster animation
                let base_tick_ms = animation::DEFAULT_TICK_RATE_MS as f64;
                let adjusted_tick_ms = (base_tick_ms * speed_factor).max(20.0) as u64;
                self.tick_rate = Duration::from_millis(adjusted_tick_ms);

                // Determine how many clusters to process based on IOPS (faster drives process more at once)
                let clusters_per_operation = (self.current_drive.iops() as usize).max(1);

                // Check if we need to select a new file to defragment
                if self.current_file_read_progress.is_none() {
                    // Chercher un bloc Pending (à défragmenter) - choix ALÉATOIRE comme dans PHP
                    let pending_indices: Vec<usize> = self.clusters
                        .iter()
                        .enumerate()
                        .filter(|&(_, c)| *c == ClusterState::Pending)
                        .map(|(i, _)| i)
                        .collect();

                    if let Some(&pending_idx) = pending_indices.choose(&mut rng) {
                        // Simulate file size - different files have different sizes
                        let file_size = 1 + (rng.gen::<usize>() % 5); // Files of 1-5 clusters

                        // Mark the first cluster in the file as Reading
                        self.clusters[pending_idx] = ClusterState::Reading;
                        self.read_pos = Some(pending_idx);

                        // Son de seek quand on change de position
                        if let Some(ref audio) = self.audio {
                            audio.play_seek();
                        }

                        // Trouver suffisamment de blocs contigus Unused pour écrire le fichier entier
                        if let Some(unused_start_idx) = self.find_contiguous_unused_clusters(file_size) {
                            // Mark the appropriate number of clusters as Writing in sequence
                            let mut write_positions = Vec::new();
                            for i in 0..file_size.min(clusters_per_operation) {
                                if unused_start_idx + i < self.clusters.len() {
                                    self.clusters[unused_start_idx + i] = ClusterState::Writing;
                                    write_positions.push(unused_start_idx + i);
                                }
                            }

                            // Set the write position to the first cluster of the file
                            self.write_pos = Some(unused_start_idx);

                            // Initialize file defragmentation progress
                            self.current_file_read_progress = Some(FileDefragPhase::Reading {
                                progress: 0
                            });
                        }
                    } else {
                        // Plus de blocs Pending, défragmentation terminée
                        self.read_pos = None;
                        self.write_pos = None;
                        self.phase = DefragPhase::Finished;
                    }
                } else {
                    // Continue the defragmentation of the current file
                    match &mut self.current_file_read_progress {
                        Some(FileDefragPhase::Reading { progress }) => {
                            // Process the current cluster being read
                            if let Some(reading_idx) = self.read_pos {
                                if self.clusters[reading_idx] == ClusterState::Reading {
                                    self.clusters[reading_idx] = ClusterState::Unused;
                                    // Son de lecture
                                    if let Some(ref audio) = self.audio {
                                        audio.play_read();
                                    }

                                    // Mark more clusters as used if we're processing a large file
                                    // Find additional clusters to process without borrowing issues
                                    let mut additional_clusters_to_process = Vec::new();
                                    let mut found_count = 0;
                                    for j in (reading_idx + 1)..self.clusters.len() {
                                        if found_count >= clusters_per_operation - 1 {
                                            break;
                                        }
                                        if self.clusters[j] == ClusterState::Reading {
                                            additional_clusters_to_process.push(j);
                                            found_count += 1;
                                        }
                                    }

                                    for next_reading_idx in additional_clusters_to_process {
                                        self.clusters[next_reading_idx] = ClusterState::Unused;
                                        if let Some(ref audio) = self.audio {
                                            audio.play_read();
                                        }
                                    }

                                    *progress += clusters_per_operation;
                                    self.current_file_read_progress = Some(FileDefragPhase::Writing {
                                        progress: *progress
                                    });
                                }
                            }
                        }
                        Some(FileDefragPhase::Writing { progress }) => {
                            // Process the current cluster being written
                            if let Some(write_idx) = self.write_pos {
                                if self.clusters[write_idx] == ClusterState::Writing {
                                    self.clusters[write_idx] = ClusterState::Used;
                                    self.stats.clusters_defragged += 1;
                                    // Son d'écriture
                                    if let Some(ref audio) = self.audio {
                                        audio.play_write();
                                    }

                                    // Mark more clusters as used if we're processing a large file
                                    for i in 1..clusters_per_operation {
                                        if write_idx + i < self.clusters.len() &&
                                           self.clusters[write_idx + i] == ClusterState::Writing {
                                            self.clusters[write_idx + i] = ClusterState::Used;
                                            self.stats.clusters_defragged += 1;
                                            if let Some(ref audio) = self.audio {
                                                audio.play_write();
                                            }
                                        }
                                    }

                                    *progress += clusters_per_operation;

                                    // Check if this file is completely written
                                    if *progress >= 5 { // Assuming average file size of 5 clusters
                                        self.current_file_read_progress = Some(FileDefragPhase::Completed);
                                    } else {
                                        self.current_file_read_progress = Some(FileDefragPhase::Reading {
                                            progress: *progress
                                        });
                                    }
                                }
                            }
                        }
                        Some(FileDefragPhase::Completed) => {
                            // Reset for the next file
                            self.current_file_read_progress = None;
                        }
                        None => {} // Shouldn't happen in this branch
                    }
                }
            }
            DefragPhase::Finished => {
                // In demo mode, restart automatically after 3 seconds
                if self.demo_mode && self.animation_step > animation::FINISH_WAIT_TICKS / 2 {
                    self.restart();
                } else if !self.demo_mode && self.animation_step > animation::FINISH_WAIT_TICKS {
                    // Normal mode: quit after waiting
                    self.running = false;
                }
            }
        }
    }

    fn handle_menu_action(&mut self) {
        match (self.selected_menu, self.selected_item) {
            (0, 0) => {
                // Begin optimization - restart defrag
                self.restart();
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

impl App {
    /// Find a sequence of contiguous unused clusters for a file of a given size
    fn find_contiguous_unused_clusters(&self, size: usize) -> Option<usize> {
        if size == 0 {
            return None;
        }

        let mut current_run = 0;
        let mut start_pos = None;

        for (i, &cluster) in self.clusters.iter().enumerate() {
            if cluster == ClusterState::Unused {
                if current_run == 0 {
                    start_pos = Some(i);
                }
                current_run += 1;

                if current_run >= size {
                    return start_pos;
                }
            } else {
                current_run = 0;
            }
        }

        None
    }

    /// Find the next cluster of a given state after a specific position
    pub fn find_next_cluster_in_file(&self, start_pos: usize, state: ClusterState) -> Option<usize> {
        for i in (start_pos + 1)..self.clusters.len() {
            if self.clusters[i] == state {
                return Some(i);
            }
        }
        None
    }
    
    /// Count clusters of a specific state
    pub fn count_clusters(&self, state: ClusterState) -> usize {
        self.clusters.iter().filter(|&&c| c == state).count()
    }
    
    /// Get fragmentation percentage (0.0 to 1.0)
    pub fn fragmentation_percent(&self) -> f32 {
        let pending = self.count_clusters(ClusterState::Pending);
        let total_data = pending + self.count_clusters(ClusterState::Used);
        if total_data == 0 {
            return 0.0;
        }
        pending as f32 / total_data as f32
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
use crate::audio::AudioEngine;
use crate::constants::{
    animation, audio as audio_const, defrag_type::DefragStyle, disk, ui as ui_const,
};
use crate::dos_files::DosFileProvider;

use crate::models::{ClusterState, DefragPhase, DefragStats};
use rand::prelude::{Rng, SliceRandom};
use std::{
    io::Result,
    sync::mpsc,
    time::{Duration, Instant},
};

// -- CLI arguments ------------------------------------------------------------

#[derive(clap::Parser)]
#[command(
    name = "defrag",
    version = "0.1.0",
    about = "MS-DOS Defragmenter Simulation"
)]
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

// -- Disk drive types ----------------------------------------------------------

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

// -- File simulation --------------------------------------------------------

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
        Self {
            clusters,
            size,
            is_fragmented,
        }
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

// -- Application state --------------------------------------------------------

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

        if let Some(s) = start {
            self.regions.push((s, length));
        }

        self.regions.sort_by(|a, b| b.1.cmp(&a.1));
        self.dirty = false;
    }

    /// Find a region with at least `size` contiguous clusters
    pub fn find_region(&self, size: usize) -> Option<usize> {
        self.regions
            .iter()
            .find(|(_, len)| *len >= size)
            .map(|(start, _)| *start)
    }
}

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub tick_rate: Duration,
    pub width: usize,
    pub height: usize,
    pub clusters: Vec<ClusterState>,
    pub stats: DefragStats,
    pub phase: DefragPhase,
    pub animation_step: u64,
    pub read_pos: Option<usize>,
    pub write_pos: Option<usize>,
    pub current_file_read_progress: Option<FileDefragPhase>,
    pub current_filename: Option<String>,
    pub current_op_end_time: Option<Instant>,
    status_message: String,
    file_provider: DosFileProvider,
    pub menu_open: bool,
    pub selected_menu: usize,
    pub selected_item: usize,
    pub show_about_box: bool,
    pub audio: Option<AudioEngine>,
    pub current_drive: DiskDrive,
    pub drive_collection: DiskDriveCollection,
    pub ui_style: DefragStyle,
    free_space_cache: FreeSpaceCache,
    pub demo_mode: bool,
    pending_indices_cache: Vec<usize>,
    pending_cache_dirty: bool,
}

impl App {
    pub fn new(
        width: usize,
        height: usize,
        fill_percent: f32,
        enable_sound: bool,
        drive_letter: char,
        ui_style: DefragStyle,
    ) -> Self {
        let total_clusters = width * height;
        let mut rng = rand::thread_rng();

        let num_pending = (total_clusters as f32 * fill_percent) as usize;
        let num_bad = (total_clusters as f32 * ui_const::BAD_BLOCK_PERCENT) as usize;

        let mut clusters: Vec<ClusterState> = Vec::with_capacity(total_clusters);

        for _ in 0..(num_pending.saturating_sub(2)) {
            clusters.push(ClusterState::Pending);
        }

        clusters.push(ClusterState::Writing);
        clusters.push(ClusterState::Reading);

        while clusters.len() < total_clusters - num_bad {
            clusters.push(ClusterState::Unused);
        }

        clusters.shuffle(&mut rng);

        let mut bad_positions: Vec<usize> = (0..clusters.len()).collect();
        bad_positions.shuffle(&mut rng);
        for &pos in bad_positions.iter().take(num_bad) {
            clusters.insert(pos.min(clusters.len()), ClusterState::Bad);
        }

        clusters.truncate(total_clusters);

        if !clusters.is_empty() {
            clusters[0] = ClusterState::Unmovable;
        }

        let total_to_defrag = clusters
            .iter()
            .filter(|&&c| c == ClusterState::Pending)
            .count()
            + 2;

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
            current_filename: None,
            current_op_end_time: None,
            status_message: "Initializing...".to_string(),
            file_provider: DosFileProvider::new(),
            menu_open: false,
            selected_menu: 0,
            selected_item: 0,
            show_about_box: false,
            audio: if enable_sound {
                let mut audio = AudioEngine::new();
                if let Some(ref mut audio_engine) = audio {
                    audio_engine.set_iops(current_drive.iops());
                }
                audio
            } else {
                None
            },
            current_drive,
            drive_collection,
            ui_style,
            free_space_cache: FreeSpaceCache::new(),
            demo_mode: false,
            pending_indices_cache: Vec::new(),
            pending_cache_dirty: true,
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.phase == DefragPhase::Defragmenting || self.phase == DefragPhase::Analyzing {
            self.paused = !self.paused;
            if self.paused {
                if let Some(ref audio) = self.audio {
                    audio.stop_all();
                }
            }
        }
    }

    pub fn toggle_demo_mode(&mut self) {
        self.demo_mode = !self.demo_mode;
    }

    pub fn restart(&mut self) {
        let mut rng = rand::thread_rng();
        let total_clusters = self.width * self.height;
        let fill_percent = ui_const::DEFAULT_FILL_PERCENT;

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
            self.clusters
                .insert(pos.min(self.clusters.len()), ClusterState::Bad);
        }
        self.clusters.truncate(total_clusters);
        if !self.clusters.is_empty() {
            self.clusters[0] = ClusterState::Unmovable;
        }

        let total_to_defrag = self
            .clusters
            .iter()
            .filter(|&&c| c == ClusterState::Pending)
            .count()
            + 2;
        self.stats = DefragStats {
            total_to_defrag,
            clusters_defragged: 0,
            start_time: Instant::now(),
        };

        self.phase = DefragPhase::Initializing;
        self.animation_step = 0;
        self.read_pos = None;
        self.write_pos = None;
        self.current_file_read_progress = None;
        self.current_filename = None;
        self.current_op_end_time = None;
        self.status_message = "Initializing...".to_string();
        self.paused = false;
        self.file_provider = DosFileProvider::new();

        self.free_space_cache.invalidate();
        self.pending_cache_dirty = true;
    }

    pub fn estimated_time_remaining(&self) -> Option<Duration> {
        if self.stats.clusters_defragged == 0 || self.phase != DefragPhase::Defragmenting {
            return None;
        }

        let elapsed = self.stats.start_time.elapsed();
        let remaining = self
            .stats
            .total_to_defrag
            .saturating_sub(self.stats.clusters_defragged);

        if remaining == 0 {
            return Some(Duration::ZERO);
        }

        let rate = self.stats.clusters_defragged as f64 / elapsed.as_secs_f64();
        if rate <= 0.0 {
            return None;
        }

        let remaining_secs = remaining as f64 / rate;
        Some(Duration::from_secs_f64(remaining_secs))
    }

    pub fn progress_percent(&self) -> f32 {
        if self.stats.total_to_defrag == 0 {
            return 100.0;
        }
        (self.stats.clusters_defragged as f32 / self.stats.total_to_defrag as f32) * 100.0
    }

    pub fn run(&mut self, term: &mut crate::ui::TuiWrapper, rx: mpsc::Receiver<()>) -> Result<()> {
        use crossterm::event::{self, Event, KeyCode, KeyEventKind};

        let mut last_tick = Instant::now();
        while self.running {
            term.draw(|frame| {
                match self.ui_style {
                    DefragStyle::Windows98 => crate::win98::render_win98_app(&self, frame),
                    DefragStyle::Windows95 => crate::win98::render_win98_app(&self, frame),
                    DefragStyle::MsDos => crate::ui::render_app(&self, frame),
                }
            })?;

            if rx.try_recv().is_ok() {
                self.running = false;
            }

            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
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
                                self.show_about_box = true;
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                if let Some(ref mut audio) = self.audio {
                                    audio.toggle();
                                } else {
                                    self.audio = AudioEngine::new();
                                }
                            }
                            KeyCode::F(10) | KeyCode::Tab => {
                                self.menu_open = !self.menu_open;
                                if self.menu_open {
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Left => {
                                if self.menu_open {
                                    self.selected_menu = if self.selected_menu == 0 {
                                        4
                                    } else {
                                        self.selected_menu - 1
                                    };
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
                                    let max_items =
                                        crate::ui::get_menu_items(self.selected_menu).len();
                                    self.selected_item = if self.selected_item == 0 {
                                        max_items.saturating_sub(1)
                                    } else {
                                        self.selected_item - 1
                                    };
                                }
                            }
                            KeyCode::Down => {
                                if self.menu_open {
                                    let max_items =
                                        crate::ui::get_menu_items(self.selected_menu).len();
                                    self.selected_item = (self.selected_item + 1) % max_items;
                                }
                            }
                            KeyCode::Enter => {
                                if self.menu_open {
                                    self.handle_menu_action();
                                    self.menu_open = false;
                                }
                            }
                            KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => {
                                if !self.menu_open {
                                    self.toggle_pause();
                                }
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                if !self.menu_open {
                                    self.restart();
                                }
                            }
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

            if last_tick.elapsed() >= self.tick_rate && !self.paused {
                self.update();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    pub fn update(&mut self) {
        self.animation_step += 1;
        self.tick_rate = Duration::from_millis(animation::DEFAULT_TICK_RATE_MS);

        if self.phase != DefragPhase::Defragmenting {
            self.status_message = self.get_phase_status().to_string();
        }


        match self.phase {
            DefragPhase::Initializing => {
                if self.animation_step > 20 {
                    self.phase = DefragPhase::Analyzing;
                    self.animation_step = 0;
                }
            }
            DefragPhase::Analyzing => {
                let total_clusters = self.width * self.height;
                let scan_pos = (self.animation_step as usize * 5).min(total_clusters - 1);
                self.read_pos = Some(scan_pos);

                if self.animation_step % 3 == 0 {
                    if let Some(ref audio) = self.audio {
                        audio.play_seek();
                    }
                }

                if self.animation_step > (total_clusters as u64 / 5) + 10 {
                    self.read_pos = None;
                    self.phase = DefragPhase::Defragmenting;
                    self.animation_step = 0;
                    self.current_op_end_time = Some(Instant::now());
                }
            }
            DefragPhase::Defragmenting => {
                if self.current_op_end_time.map_or(true, |t| Instant::now() >= t) {
                    let mut rng = rand::thread_rng();
                    let clusters_per_operation = (self.current_drive.iops() as usize).max(1);

                    if self.current_file_read_progress.is_none() {
                        let pending_indices: Vec<usize> = self
                            .clusters
                            .iter()
                            .enumerate()
                            .filter(|&(_, c)| *c == ClusterState::Pending)
                            .map(|(i, _)| i)
                            .collect();

                        if let Some(pending_idx) = pending_indices.choose(&mut rng).copied() {
                            self.current_filename = self.file_provider.get_random_filename();
                            let file_size = rng.gen_range(1..=5);

                             let base_duration_ms = rng.gen_range(1000..=3000);
                             let iops_factor = self.current_drive.iops().max(1) as f64;
                             let final_duration = Duration::from_millis((base_duration_ms as f64 / iops_factor) as u64);
                             self.current_op_end_time = Some(Instant::now() + final_duration);

                            self.clusters[pending_idx] = ClusterState::Reading;
                            self.read_pos = Some(pending_idx);
                            if let Some(ref audio) = self.audio {
                                audio.play_seek();
                            }

                            if let Some(unused_start_idx) = self.find_contiguous_unused_clusters(file_size) {
                                for i in 0..file_size.min(clusters_per_operation) {
                                    if unused_start_idx + i < self.clusters.len() {
                                        self.clusters[unused_start_idx + i] = ClusterState::Writing;
                                    }
                                }
                                self.write_pos = Some(unused_start_idx);
                                self.current_file_read_progress = Some(FileDefragPhase::Reading { progress: 0 });
                                self.status_message = format!(
                                    "Reading {}...",
                                    self.current_filename.as_deref().unwrap_or("file")
                                );

                            } else {
                                self.clusters[pending_idx] = ClusterState::Used;
                                self.stats.clusters_defragged += 1;
                                self.read_pos = None;
                                self.current_filename = None;
                                if let Some(ref audio) = self.audio {
                                    audio.play_write();
                                }
                                self.current_op_end_time = Some(Instant::now());
                            }
                        } else {
                            self.phase = DefragPhase::Finished;
                            self.current_filename = None;
                            self.read_pos = None;
                            self.write_pos = None;
                        }
                    } else {
                        match &mut self.current_file_read_progress {
                            Some(FileDefragPhase::Reading { .. }) => {
                                if let Some(reading_idx) = self.read_pos {
                                    if self.clusters[reading_idx] == ClusterState::Reading {
                                        self.clusters[reading_idx] = ClusterState::Unused;
                                        if let Some(ref audio) = self.audio {
                                            audio.play_read();
                                        }
                                    }
                                }
                                self.current_file_read_progress = Some(FileDefragPhase::Writing { progress: 0 });
                                self.status_message = format!(
                                    "Writing {}...",
                                    self.current_filename.as_deref().unwrap_or("file")
                                );
                            }
                            Some(FileDefragPhase::Writing { .. }) => {
                                if let Some(write_idx) = self.write_pos {
                                    if self.clusters[write_idx] == ClusterState::Writing {
                                        self.clusters[write_idx] = ClusterState::Used;
                                        self.stats.clusters_defragged += 1;
                                        if let Some(ref audio) = self.audio {
                                            audio.play_write();
                                        }
                                    }
                                }
                                self.current_file_read_progress = Some(FileDefragPhase::Completed);
                                self.status_message = format!(
                                    "Finishing {}...",
                                    self.current_filename.as_deref().unwrap_or("file")
                                );
                            }
                            Some(FileDefragPhase::Completed) => {
                                self.current_file_read_progress = None;
                                self.current_filename = None;
                                self.current_op_end_time = Some(Instant::now());
                                self.status_message = "Looking for next file...".to_string();

                            }
                            None => {}
                        }
                    }
                }
                 else {
                    let dots = ".".repeat(((self.animation_step % 4)) as usize);
                    let base_message = match self.current_file_read_progress {
                        Some(FileDefragPhase::Reading { .. }) => "Reading",
                        Some(FileDefragPhase::Writing { .. }) => "Writing",
                        _ => "Processing",
                    };
                    self.status_message = format!(
                        "{} {}{}",
                        base_message,
                        self.current_filename.as_deref().unwrap_or("file"),
                        dots
                    );
                }
            }
            DefragPhase::Finished => {
                if self.demo_mode && self.animation_step > animation::FINISH_WAIT_TICKS / 2 {
                    self.restart();
                } else if !self.demo_mode && self.animation_step > animation::FINISH_WAIT_TICKS {
                    self.running = false;
                }
            }
        }
    }

    fn handle_menu_action(&mut self) {
        match (self.selected_menu, self.selected_item) {
            (0, 0) => {
                self.restart();
            }
            (0, 4) => {
                self.running = false;
            }
            (1, 0) => {
                if self.phase != DefragPhase::Analyzing {
                    self.phase = DefragPhase::Analyzing;
                    self.animation_step = 0;
                }
            }
            (4, 0) | (4, 1) => {
                self.show_about_box = true;
            }
            _ => {}
        }
    }
}

impl App {
    fn find_contiguous_unused_clusters(&mut self, size: usize) -> Option<usize> {
        if size == 0 {
            return None;
        }

        let mut current_run = 0;
        let mut start_pos: Option<usize> = None;

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
                start_pos = None;
            }
        }

        None
    }

    fn invalidate_caches(&mut self) {
        self.free_space_cache.invalidate();
        self.pending_cache_dirty = true;
    }

    fn get_pending_indices(&mut self) -> &[usize] {
        if self.pending_cache_dirty {
            self.pending_indices_cache = self
                .clusters
                .iter()
                .enumerate()
                .filter(|&(_, c)| *c == ClusterState::Pending)
                .map(|(i, _)| i)
                .collect();
            self.pending_cache_dirty = false;
        }
        &self.pending_indices_cache
    }

    fn find_next_cluster_in_file(
        &self,
        start_pos: usize,
        state: ClusterState,
    ) -> Option<usize> {
        for i in (start_pos + 1)..self.clusters.len() {
            if self.clusters[i] == state {
                return Some(i);
            }
        }
        None
    }

    pub fn count_clusters(&self, state: ClusterState) -> usize {
        self.clusters.iter().filter(|&&c| c == state).count()
    }

    pub fn fragmentation_percent(&self) -> f32 {
        let pending = self.count_clusters(ClusterState::Pending);
        let total_data = pending + self.count_clusters(ClusterState::Used);
        if total_data == 0 {
            return 0.0;
        }
        pending as f32 / total_data as f32
    }

    fn get_phase_status(&self) -> &'static str {
        match self.phase {
            DefragPhase::Initializing => "Initializing...",
            DefragPhase::Analyzing => "Analyzing disk...",
            DefragPhase::Defragmenting => "Defragmenting...",
            DefragPhase::Finished => "Complete",
        }
    }


    pub fn status_text(&self) -> &str {
        if self.paused {
            return "Paused";
        }
        &self.status_message
    }
}

pub fn parse_size(size_str: &str) -> Result<(usize, usize)> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Size must be in format WxH",
        ));
    }
    let width: usize = parts[0]
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid width"))?;
    let height: usize = parts[1]
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid height"))?;
    Ok((width, height))
}

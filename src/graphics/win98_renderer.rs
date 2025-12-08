//! Windows 98 Disk Defragmenter Graphical Renderer
//! Faithful recreation of the Win98 defrag interface using SDL2

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use super::sdl_backend::{colors, SdlBackend, SdlConfig, SdlEvent};
use super::win98_widgets::{Button, ButtonState, ProgressBar, SunkenPanel, Win98WindowWidget};
use super::ResourceCache;
use crate::app::App;
use crate::models::{ClusterState, DefragPhase};

/// Cluster size in pixels for the disk grid
const CLUSTER_SIZE: u32 = 8;

/// Spacing between clusters (gap-px in CSS = 1px)
const CLUSTER_GAP: u32 = 1;

/// Win98 cluster states (matching the JavaScript implementation)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Win98ClusterState {
    NotDefragmented,  // Navy blue
    InProgress,       // Red
    Completed,        // Cyan
}

impl Win98ClusterState {
    pub fn color(&self) -> Color {
        match self {
            Win98ClusterState::NotDefragmented => colors::DEFRAG_IDLE,
            Win98ClusterState::InProgress => colors::DEFRAG_PROGRESS,
            Win98ClusterState::Completed => colors::DEFRAG_DONE,
        }
    }
}

impl From<&ClusterState> for Win98ClusterState {
    fn from(state: &ClusterState) -> Self {
        match state {
            ClusterState::Used => Win98ClusterState::Completed,
            ClusterState::Pending => Win98ClusterState::NotDefragmented,
            ClusterState::Reading | ClusterState::Writing => Win98ClusterState::InProgress,
            ClusterState::Unused | ClusterState::Bad | ClusterState::Unmovable => Win98ClusterState::NotDefragmented,
        }
    }
}

/// The main Win98 graphical renderer
pub struct Win98GraphicalRenderer {
    backend: SdlBackend,
    resource_cache: ResourceCache,
    // UI State
    window_widget: Win98WindowWidget,
    settings_button: Button,
    start_pause_button: Button,
    stop_button: Button,
    progress_bar: ProgressBar,
    disk_panel: SunkenPanel,
    // Mouse state
    mouse_x: i32,
    mouse_y: i32,
}

impl Win98GraphicalRenderer {
    /// Create a new Win98 graphical renderer
    pub fn new() -> Result<Self, String> {
        let config = SdlConfig {
            width: 640,
            height: 480,
            title: "Disk Defragmenter".to_string(),
            scale: 1,
        };
        
        let backend = SdlBackend::new(config)?;
        
        // Calculate window position (centered)
        let window_width = 500;
        let window_height = 380;
        let window_x = (640 - window_width) / 2;
        let window_y = (480 - window_height) / 2;
        
        let window_widget = Win98WindowWidget::new(
            window_x as i32,
            window_y as i32,
            window_width,
            window_height,
            "Disk Defragmenter",
        );
        
        let client = window_widget.client_area();
        
        // Disk panel (takes most of the space)
        let disk_panel = SunkenPanel::new(
            client.x + 8,
            client.y + 8,
            client.width - 16,
            client.height - 120,
        );
        
        // Legend area is below the disk panel
        let legend_y = disk_panel.area.y + disk_panel.area.height as i32 + 12;
        
        // Progress bar
        let progress_bar = ProgressBar::new(
            client.x + 8,
            legend_y + 24,
            client.width - 16,
            16,
        );
        
        // Buttons
        let button_y = progress_bar.area.y + progress_bar.area.height as i32 + 32;
        
        let settings_button = Button::new(
            client.x + 8,
            button_y,
            85,
            23,
            "Settings...",
        );
        
        let start_pause_button = Button::new(
            client.x + client.width as i32 - 180,
            button_y,
            75,
            23,
            "Start",
        );
        
        let stop_button = Button::new(
            client.x + client.width as i32 - 90,
            button_y,
            75,
            23,
            "Stop",
        );
        
        // Create and initialize the resource cache with all needed images
        let mut resource_cache = ResourceCache::new();

        // Load all the UI images
        let _ = resource_cache.load_image_from_file("cluster_sprites", "static/imgs/cluster_sprites.png");
        let _ = resource_cache.load_image_from_file("border_left", "static/imgs/border_left.png");
        let _ = resource_cache.load_image_from_file("right_scroll_bar", "static/imgs/right_scroll_bar.png");
        let _ = resource_cache.load_image_from_file("title_left", "static/imgs/title_left.png");
        let _ = resource_cache.load_image_from_file("title_right", "static/imgs/title_right.png");
        let _ = resource_cache.load_image_from_file("title_bg", "static/imgs/title_bg.png");
        let _ = resource_cache.load_image_from_file("top_right_scroll", "static/imgs/top_right_scroll.png");

        Ok(Self {
            backend,
            resource_cache,
            window_widget,
            settings_button,
            start_pause_button,
            stop_button,
            progress_bar,
            disk_panel,
            mouse_x: 0,
            mouse_y: 0,
        })
    }
    
    /// Main run loop for the graphical renderer
    pub fn run(&mut self, app: &mut App) -> Result<(), String> {
        let target_fps = 60;
        let frame_duration = Duration::from_micros(1_000_000 / target_fps);
        
        while self.backend.is_running() && app.running {
            let frame_start = Instant::now();
            
            // Process events
            self.handle_events(app);
            
            // Update application state
            app.update();
            
            // Update UI state from app
            self.update_ui_state(app);
            
            // Render
            self.render(app);
            
            // Cap frame rate
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        
        Ok(())
    }
    
    /// Render a single frame
    fn render(&mut self, app: &App) {
        // Clear with desktop color
        self.backend.clear();
        
        // Draw window
        self.window_widget.draw(&mut self.backend.canvas);
        
        // Draw title bar text
        self.draw_title_text();
        
        // Draw disk panel
        self.disk_panel.draw(&mut self.backend.canvas);
        
        // Draw disk grid
        self.draw_disk_grid(app);
        
        // Draw legend
        self.draw_legend();
        
        // Draw progress bar
        self.progress_bar.draw(&mut self.backend.canvas);
        
        // Draw progress text
        self.draw_progress_text(app);
        
        // Draw buttons
        self.settings_button.draw(&mut self.backend.canvas);
        self.start_pause_button.draw(&mut self.backend.canvas);
        self.stop_button.draw(&mut self.backend.canvas);
        
        // Draw button text
        self.draw_button_text();
        
        // Present
        self.backend.present();
    }
    
    /// Handle SDL events
    fn handle_events(&mut self, app: &mut App) {
        let events = self.backend.poll_events();
        
        for event in events {
            match event {
                SdlEvent::Quit => {
                    app.running = false;
                }
                SdlEvent::KeyDown(keycode) => {
                    self.handle_keydown(app, keycode);
                }
                SdlEvent::MouseMove { x, y } => {
                    self.mouse_x = x;
                    self.mouse_y = y;
                    self.update_button_hover();
                }
                SdlEvent::MouseDown { x, y, .. } => {
                    self.handle_mouse_down(app, x, y);
                }
                SdlEvent::MouseUp { x, y, .. } => {
                    self.handle_mouse_up(app, x, y);
                }
                _ => {}
            }
        }
    }
    
    /// Handle keyboard input
    fn handle_keydown(&mut self, app: &mut App, keycode: Keycode) {
        match keycode {
            Keycode::Escape | Keycode::Q => {
                app.running = false;
            }
            Keycode::Space | Keycode::Return => {
                // Toggle start/pause
                self.toggle_defrag(app);
            }
            Keycode::S => {
                // Toggle sound
                if let Some(ref mut audio) = app.audio {
                    audio.toggle();
                }
            }
            _ => {}
        }
    }
    
    /// Update button hover states
    fn update_button_hover(&mut self) {
        // Settings button
        if self.settings_button.area.contains(self.mouse_x, self.mouse_y) {
            if self.settings_button.state != ButtonState::Pressed {
                self.settings_button.state = ButtonState::Hovered;
            }
        } else if self.settings_button.state == ButtonState::Hovered {
            self.settings_button.state = ButtonState::Normal;
        }
        
        // Start/Pause button
        if self.start_pause_button.area.contains(self.mouse_x, self.mouse_y) {
            if self.start_pause_button.state != ButtonState::Pressed {
                self.start_pause_button.state = ButtonState::Hovered;
            }
        } else if self.start_pause_button.state == ButtonState::Hovered {
            self.start_pause_button.state = ButtonState::Normal;
        }
        
        // Stop button
        if self.stop_button.area.contains(self.mouse_x, self.mouse_y) {
            if self.stop_button.state != ButtonState::Pressed && self.stop_button.state != ButtonState::Disabled {
                self.stop_button.state = ButtonState::Hovered;
            }
        } else if self.stop_button.state == ButtonState::Hovered {
            self.stop_button.state = ButtonState::Normal;
        }
    }
    
    /// Handle mouse button down
    fn handle_mouse_down(&mut self, app: &mut App, x: i32, y: i32) {
        // Play mouse down sound
        if let Some(ref audio) = app.audio {
            audio.play_mouse_down();
        }
        
        if self.settings_button.area.contains(x, y) {
            self.settings_button.state = ButtonState::Pressed;
        } else if self.start_pause_button.area.contains(x, y) {
            self.start_pause_button.state = ButtonState::Pressed;
        } else if self.stop_button.area.contains(x, y) && self.stop_button.state != ButtonState::Disabled {
            self.stop_button.state = ButtonState::Pressed;
        }
    }
    
    /// Handle mouse button up
    fn handle_mouse_up(&mut self, app: &mut App, x: i32, y: i32) {
        // Play mouse up sound
        if let Some(ref audio) = app.audio {
            audio.play_mouse_up();
        }
        
        // Check for button clicks
        if self.settings_button.state == ButtonState::Pressed {
            self.settings_button.state = ButtonState::Normal;
            if self.settings_button.area.contains(x, y) {
                // Settings clicked - TODO: show settings dialog
            }
        }
        
        if self.start_pause_button.state == ButtonState::Pressed {
            self.start_pause_button.state = ButtonState::Normal;
            if self.start_pause_button.area.contains(x, y) {
                self.toggle_defrag(app);
            }
        }
        
        if self.stop_button.state == ButtonState::Pressed {
            self.stop_button.state = ButtonState::Normal;
            if self.stop_button.area.contains(x, y) {
                self.stop_defrag(app);
            }
        }
    }
    
    /// Toggle between start/pause
    fn toggle_defrag(&mut self, app: &mut App) {
        match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => {
                // Start
                app.phase = DefragPhase::Analyzing;
                app.animation_step = 0;
            }
            DefragPhase::Analyzing | DefragPhase::Defragmenting => {
                // Pause - for now, just stop
                // TODO: implement proper pause
            }
        }
    }
    
    /// Stop defragmentation
    fn stop_defrag(&mut self, app: &mut App) {
        app.phase = DefragPhase::Finished;
    }
    
    /// Update UI state based on app state
    fn update_ui_state(&mut self, app: &App) {
        // Update window title
        self.window_widget.title = match app.phase {
            DefragPhase::Defragmenting => format!("Defragmenting Drive {}", app.current_drive.letter()),
            DefragPhase::Analyzing => format!("Defragmenting Drive {} (analyzing)", app.current_drive.letter()),
            _ => "Disk Defragmenter".to_string(),
        };
        
        // Update button text
        self.start_pause_button.text = match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => "Start".to_string(),
            DefragPhase::Analyzing | DefragPhase::Defragmenting => "Pause".to_string(),
        };
        
        // Update stop button state
        self.stop_button.state = match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => ButtonState::Disabled,
            _ => if self.stop_button.area.contains(self.mouse_x, self.mouse_y) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            },
        };
        
        // Update progress bar
        let progress = if app.stats.total_to_defrag > 0 {
            app.stats.clusters_defragged as f64 / app.stats.total_to_defrag as f64
        } else {
            0.0
        };
        self.progress_bar.set_progress(progress);
    }
    
    /// Draw the disk cluster grid
    fn draw_disk_grid(&mut self, app: &App) {
        let inner = self.disk_panel.inner_area();

        // Calculate grid dimensions
        let cols = (inner.width / (CLUSTER_SIZE + CLUSTER_GAP)) as usize;
        let rows = (inner.height / (CLUSTER_SIZE + CLUSTER_GAP)) as usize;

        // For now, use the simple colored rectangles approach
        // The texture implementation needs to be restructured to work with SDL2 lifetimes
        for (i, cluster) in app.clusters.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;

            if row >= rows {
                break;
            }

            let x = inner.x + (col as u32 * (CLUSTER_SIZE + CLUSTER_GAP)) as i32;
            let y = inner.y + (row as u32 * (CLUSTER_SIZE + CLUSTER_GAP)) as i32;

            // Get color based on cluster state
            let win98_state = Win98ClusterState::from(cluster);
            let color = win98_state.color();

            self.backend.fill_rect(x, y, CLUSTER_SIZE, CLUSTER_SIZE, color);
        }
    }
    
    /// Draw the legend (Not defragmented, In progress, Defragmented)
    fn draw_legend(&mut self) {
        let legend_y = self.disk_panel.area.y + self.disk_panel.area.height as i32 + 8;
        let client = self.window_widget.client_area();
        
        // Calculate positions for three legend items
        let item_width = (client.width / 3) as i32;
        
        // Not defragmented (navy)
        let x1 = client.x + 16;
        self.backend.fill_rect(x1, legend_y, 12, 12, colors::DEFRAG_IDLE);
        let _ = self.backend.draw_text("Not defragmented", x1 + 16, legend_y - 1, 11, colors::TEXT);
        
        // In progress (red)
        let x2 = client.x + item_width + 16;
        self.backend.fill_rect(x2, legend_y, 12, 12, colors::DEFRAG_PROGRESS);
        let _ = self.backend.draw_text("In progress", x2 + 16, legend_y - 1, 11, colors::TEXT);
        
        // Defragmented (cyan)
        let x3 = client.x + item_width * 2 + 16;
        self.backend.fill_rect(x3, legend_y, 12, 12, colors::DEFRAG_DONE);
        let _ = self.backend.draw_text("Defragmented", x3 + 16, legend_y - 1, 11, colors::TEXT);
    }
    
    /// Draw progress text
    fn draw_progress_text(&mut self, app: &App) {
        let progress = if app.stats.total_to_defrag > 0 {
            (app.stats.clusters_defragged as f64 / app.stats.total_to_defrag as f64 * 100.0) as u32
        } else {
            if app.phase == DefragPhase::Finished { 100 } else { 0 }
        };
        
        let y = self.progress_bar.area.y - 18;

        // Status text on the left
        let status_text = if let Some(filename) = &app.current_filename {
            let max_len = 45;
            let display_name = if filename.len() > max_len { &filename[..max_len] } else { filename };
            format!("Defragmenting: {}", display_name)
        } else {
            match app.phase {
                DefragPhase::Analyzing => "Analyzing drive...".to_string(),
                DefragPhase::Finished => "Defragmentation is 100% complete.".to_string(),
                _ => "".to_string(),
            }
        };
        let _ = self.backend.draw_text(&status_text, self.progress_bar.area.x, y, 13, colors::TEXT);
        
        // Percentage text on the right
        let percent_text = format!("{}% complete", progress);
        if let Ok(text_width) = self.backend.get_text_width(&percent_text, 13) {
            let x_right = self.progress_bar.area.x + self.progress_bar.area.width as i32 - text_width as i32;
            let _ = self.backend.draw_text(&percent_text, x_right, y, 13, colors::TEXT);
        }
    }
    
    /// Draw button text
    fn draw_button_text(&mut self) {
        // Settings button
        let _ = self.backend.draw_text_centered(
            &self.settings_button.text,
            self.settings_button.area.x,
            self.settings_button.area.y + 4,
            self.settings_button.area.width,
            13,
            colors::TEXT,
        );
        
        // Start/Pause button
        let _ = self.backend.draw_text_centered(
            &self.start_pause_button.text,
            self.start_pause_button.area.x,
            self.start_pause_button.area.y + 4,
            self.start_pause_button.area.width,
            13,
            colors::TEXT,
        );
        
        // Stop button
        let stop_color = if self.stop_button.state == ButtonState::Disabled {
            colors::BUTTON_SHADOW
        } else {
            colors::TEXT
        };
        let _ = self.backend.draw_text_centered(
            &self.stop_button.text,
            self.stop_button.area.x,
            self.stop_button.area.y + 4,
            self.stop_button.area.width,
            13,
            stop_color,
        );
    }
    
    /// Draw title bar text
    fn draw_title_text(&mut self) {
        let title_area = self.window_widget.title_bar_area();
        let _ = self.backend.draw_text(
            &self.window_widget.title,
            title_area.x + 4,
            title_area.y + 2,
            14,
            colors::WHITE,
        );
    }
}

/// Run the Win98 graphical interface
pub fn run_win98_graphical(app: &mut App) -> Result<(), String> {
    let mut renderer = Win98GraphicalRenderer::new()?;
    renderer.run(app)
}

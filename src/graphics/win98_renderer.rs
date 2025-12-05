//! Windows 98 Disk Defragmenter Graphical Renderer
//! Faithful recreation of the Win98 defrag interface using SDL2

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use super::sdl_backend::{colors, SdlBackend, SdlConfig, SdlEvent};
use super::win98_widgets::{Button, ButtonState, ProgressBar, SunkenPanel, Win98WindowWidget};
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
        
        Ok(Self {
            backend,
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
    
    /// Main render function
    fn render(&mut self, app: &App) {
        // Clear with desktop color
        self.backend.clear();
        
        // Draw window
        self.window_widget.draw(&mut self.backend.canvas);
        
        // Draw disk panel
        self.disk_panel.draw(&mut self.backend.canvas);
        
        // Draw disk grid
        self.draw_disk_grid(app);
        
        // Draw legend
        self.draw_legend(app);
        
        // Draw progress bar
        self.progress_bar.draw(&mut self.backend.canvas);
        
        // Draw progress text
        self.draw_progress_text(app);
        
        // Draw buttons
        self.settings_button.draw(&mut self.backend.canvas);
        self.start_pause_button.draw(&mut self.backend.canvas);
        self.stop_button.draw(&mut self.backend.canvas);
        
        // Draw button text (we need TTF for proper text, using placeholders)
        self.draw_button_text();
        
        // Present
        self.backend.present();
    }
    
    /// Draw the disk cluster grid
    fn draw_disk_grid(&mut self, app: &App) {
        let inner = self.disk_panel.inner_area();
        
        // Calculate grid dimensions
        let cols = (inner.width / (CLUSTER_SIZE + CLUSTER_GAP)) as usize;
        let rows = (inner.height / (CLUSTER_SIZE + CLUSTER_GAP)) as usize;
        
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
    fn draw_legend(&mut self, _app: &App) {
        let legend_y = self.disk_panel.area.y + self.disk_panel.area.height as i32 + 8;
        let client = self.window_widget.client_area();
        
        // Calculate positions for three legend items
        let item_width = (client.width / 3) as i32;
        
        // Not defragmented (navy)
        let x1 = client.x + item_width / 2 - 50;
        self.backend.fill_rect(x1, legend_y, 12, 12, colors::DEFRAG_IDLE);
        
        // In progress (red)
        let x2 = client.x + item_width + item_width / 2 - 40;
        self.backend.fill_rect(x2, legend_y, 12, 12, colors::DEFRAG_PROGRESS);
        
        // Defragmented (cyan)
        let x3 = client.x + item_width * 2 + item_width / 2 - 40;
        self.backend.fill_rect(x3, legend_y, 12, 12, colors::DEFRAG_DONE);
    }
    
    /// Draw progress text (without proper font, we'll skip for now)
    fn draw_progress_text(&mut self, _app: &App) {
        // TODO: Use TTF font to draw "XX% completed" text
        // For now, the progress bar shows the progress visually
    }
    
    /// Draw button text (placeholder - needs TTF)
    fn draw_button_text(&mut self) {
        // TODO: Use TTF font to render button labels
        // For now, buttons show their state through borders
    }
}

/// Run the Win98 graphical interface
pub fn run_win98_graphical(app: &mut App) -> Result<(), String> {
    let mut renderer = Win98GraphicalRenderer::new()?;
    renderer.run(app)
}

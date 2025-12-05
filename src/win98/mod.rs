//! Windows 98 Disk Defragmenter UI implementation
//! This module recreates the Windows 98 interface in the terminal using ratatui
//! Based on the defrag98.com web implementation

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, BorderType, Widget},
};
use crate::models::{ClusterState, DefragPhase};
use crate::app::App;

// =============================================================================
// Windows 98 Color Scheme (from CSS variables)
// =============================================================================

/// Windows 98 color constants matching the CSS
pub mod colors {
    use ratatui::style::Color;
    
    // Window chrome colors
    pub const SURFACE: Color = Color::Rgb(192, 192, 192);           // --color-surface: silver
    pub const BUTTON_FACE: Color = Color::Rgb(223, 223, 223);       // --color-button-face: #dfdfdf
    pub const BUTTON_HIGHLIGHT: Color = Color::White;               // --color-button-highlight: #fff
    pub const BUTTON_SHADOW: Color = Color::Rgb(128, 128, 128);     // --color-button-shadow: gray
    pub const WINDOW_FRAME: Color = Color::Rgb(10, 10, 10);         // --color-window-frame: #0a0a0a
    
    // Title bar gradient colors
    pub const DIALOG_BLUE: Color = Color::Rgb(0, 0, 128);           // --color-dialog-blue: navy
    pub const DIALOG_BLUE_LIGHT: Color = Color::Rgb(16, 132, 208);  // --color-dialog-blue-light: #1084d0
    pub const DIALOG_GRAY: Color = Color::Rgb(128, 128, 128);       // --color-dialog-gray: gray (inactive)
    
    // Defrag specific colors (exact CSS values)
    pub const DEFRAG_IDLE: Color = Color::Rgb(0, 0, 128);           // --color-defrag-idle: navy (NOT_DEFRAGMENTED)
    pub const DEFRAG_PROGRESS: Color = Color::Rgb(255, 0, 0);       // --color-defrag-progress: red (IN_PROGRESS)
    pub const DEFRAG_DONE: Color = Color::Rgb(19, 250, 251);        // --color-defrag-done: #13fafb (COMPLETED)
    
    // Text color
    pub const TEXT: Color = Color::Rgb(34, 34, 34);                 // --color-text: #222
    
    // Desktop background
    pub const DESKTOP_TEAL: Color = Color::Rgb(0, 128, 128);        // bg-[#008080] teal
}

// =============================================================================
// Windows 98 Cluster State (matching JS enum)
// =============================================================================

/// Win98 cluster states matching the JavaScript implementation
/// const l={NOT_DEFRAGMENTED:0,IN_PROGRESS:1,COMPLETED:2}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Win98ClusterState {
    NotDefragmented,  // 0 - Navy blue (idle)
    InProgress,       // 1 - Red (currently being processed)
    Completed,        // 2 - Cyan (done)
}

impl Win98ClusterState {
    /// Get the background color for this cluster state (matching CSS classes)
    pub fn color(&self) -> Color {
        match self {
            Win98ClusterState::NotDefragmented => colors::DEFRAG_IDLE,    // bg-defrag-idle
            Win98ClusterState::InProgress => colors::DEFRAG_PROGRESS,     // bg-defrag-progress
            Win98ClusterState::Completed => colors::DEFRAG_DONE,          // bg-defrag-done
        }
    }
    
    /// Get the label for the legend
    pub fn label(&self) -> &'static str {
        match self {
            Win98ClusterState::NotDefragmented => "Not defragmented",
            Win98ClusterState::InProgress => "In progress",
            Win98ClusterState::Completed => "Defragmented",
        }
    }
}

/// Convert MS-DOS cluster state to Win98 cluster state
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

// =============================================================================
// Windows 98 Window Component
// =============================================================================

/// Renders the Windows 98 Disk Defragmenter interface
pub struct Win98Window;

impl Win98Window {
    /// Main render function - draws the complete Win98 defrag window
    pub fn render(f: &mut Frame, app: &App, area: Rect) {
        // Main window with Win98 styling (box-shadow simulation with borders)
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default()
                .bg(colors::SURFACE)
                .fg(colors::WINDOW_FRAME));
        
        let inner_area = outer_block.inner(area);
        f.render_widget(outer_block, area);
        
        // Layout the window content
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1),  // Title bar
                Constraint::Length(1),  // Window body margin top
                Constraint::Min(8),     // Disk grid (sunken panel)
                Constraint::Length(1),  // Spacing
                Constraint::Length(1),  // Legend
                Constraint::Length(1),  // Spacing  
                Constraint::Length(1),  // Progress bar
                Constraint::Length(1),  // Progress text
                Constraint::Length(1),  // Spacing
                Constraint::Length(1),  // Buttons
            ])
            .split(inner_area);
        
        // Draw components
        Self::draw_title_bar(f, app, layout[0]);
        Self::draw_disk_grid(f, app, layout[2]);
        Self::draw_legend(f, layout[4]);
        Self::draw_progress_bar(f, app, layout[6]);
        Self::draw_progress_text(f, app, layout[7]);
        Self::draw_buttons(f, app, layout[9]);
    }
    
    /// Draw the Win98 title bar with gradient effect (simulated)
    fn draw_title_bar(f: &mut Frame, app: &App, area: Rect) {
        // Simulate the gradient by using the darker blue
        // In Win98: background: linear-gradient(90deg, navy, #1084d0)
        let title_text = Self::get_title_text(app);
        
        let title = Paragraph::new(vec![
            Line::from(vec![
                Span::raw(" "),
                // Window icon placeholder
                Span::styled("▣", Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(title_text, Style::default().fg(Color::White).bold()),
            ])
        ])
        .style(Style::default().bg(colors::DIALOG_BLUE));
        
        f.render_widget(title, area);
        
        // Draw window controls on the right
        Self::draw_window_controls(f, area);
    }
    
    /// Get the window title based on current state (matching JS: G function)
    fn get_title_text(app: &App) -> String {
        match app.phase {
            DefragPhase::Defragmenting => format!("Defragmenting Drive {}", app.current_drive.letter()),
            DefragPhase::Analyzing => format!("Defragmenting Drive {} (analyzing)", app.current_drive.letter()),
            DefragPhase::Initializing => "Disk Defragmenter".to_string(),
            DefragPhase::Finished => "Disk Defragmenter".to_string(),
        }
    }
    
    /// Draw window control buttons (minimize, maximize, close)
    fn draw_window_controls(f: &mut Frame, area: Rect) {
        // Position controls at the right edge
        let controls_width = 7;
        if area.width < controls_width {
            return;
        }
        
        let controls_area = Rect {
            x: area.x + area.width - controls_width,
            y: area.y,
            width: controls_width,
            height: 1,
        };
        
        // Draw minimize, maximize/restore, close buttons (Win98 style)
        let controls = Paragraph::new(" _ □ ×")
            .style(Style::default()
                .fg(Color::Black)
                .bg(colors::BUTTON_FACE));
        
        f.render_widget(controls, controls_area);
    }
    
    /// Draw the sunken panel with disk grid
    fn draw_disk_grid(f: &mut Frame, app: &App, area: Rect) {
        // Sunken panel effect: CSS class "sunken-panel"
        // border-image with groove effect
        let panel = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(colors::BUTTON_SHADOW))
            .style(Style::default().bg(Color::Black));
        
        let grid_area = panel.inner(area);
        f.render_widget(panel, area);
        
        // Render the cluster grid
        let grid_widget = Win98DiskGrid { clusters: &app.clusters };
        f.render_widget(grid_widget, grid_area);
    }
    
    /// Draw the legend with 3 colored squares (matching JS: C component)
    fn draw_legend(f: &mut Frame, area: Rect) {
        // Layout: "flex justify-around gap-4" in CSS
        let legend_spans = vec![
            // Not defragmented (navy)
            Span::styled("■", Style::default().fg(colors::DEFRAG_IDLE)),
            Span::styled(" Not defragmented   ", Style::default().fg(colors::TEXT).bg(colors::SURFACE)),
            // In progress (red)  
            Span::styled("■", Style::default().fg(colors::DEFRAG_PROGRESS)),
            Span::styled(" In progress   ", Style::default().fg(colors::TEXT).bg(colors::SURFACE)),
            // Defragmented (cyan)
            Span::styled("■", Style::default().fg(colors::DEFRAG_DONE)),
            Span::styled(" Defragmented", Style::default().fg(colors::TEXT).bg(colors::SURFACE)),
        ];
        
        let legend = Paragraph::new(Line::from(legend_spans))
            .style(Style::default().bg(colors::SURFACE))
            .alignment(Alignment::Center);
        
        f.render_widget(legend, area);
    }
    
    /// Draw the Win98-style progress bar
    fn draw_progress_bar(f: &mut Frame, app: &App, area: Rect) {
        let progress = Self::calculate_progress(app);
        
        // Win98 progress bar: border-2 border-[#808080_#ffffff_#ffffff_#808080] bg-white
        // Fill: bg-defrag-idle (navy)
        let progress_bar = Win98ProgressBar { 
            progress,
            fill_color: colors::DEFRAG_IDLE,
        };
        
        f.render_widget(progress_bar, area);
    }
    
    /// Draw progress percentage text
    fn draw_progress_text(f: &mut Frame, app: &App, area: Rect) {
        let progress = Self::calculate_progress(app);
        let text = format!("{}% completed", (progress * 100.0) as u32);
        
        let progress_text = Paragraph::new(text)
            .style(Style::default().fg(colors::TEXT).bg(colors::SURFACE))
            .alignment(Alignment::Center);
        
        f.render_widget(progress_text, area);
    }
    
    /// Calculate progress percentage (matching JS: F selector)
    fn calculate_progress(app: &App) -> f64 {
        if app.stats.total_to_defrag == 0 {
            return 1.0;
        }
        app.stats.clusters_defragged as f64 / app.stats.total_to_defrag as f64
    }
    
    /// Draw control buttons (Settings, Start/Pause, Stop)
    fn draw_buttons(f: &mut Frame, app: &App, area: Rect) {
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),   // Left margin
                Constraint::Length(12),  // Settings button
                Constraint::Min(1),      // Flexible space
                Constraint::Length(10),  // Start/Pause button
                Constraint::Length(1),   // Gap
                Constraint::Length(10),  // Stop button
                Constraint::Length(2),   // Right margin
            ])
            .split(area);
        
        // Settings button (left side)
        Self::draw_button(f, "Settings...", button_layout[1], false);
        
        // Determine Start/Pause/Resume button text based on state
        let (primary_text, _primary_disabled) = match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => ("Start", false),
            DefragPhase::Analyzing | DefragPhase::Defragmenting => ("Pause", false),
        };
        
        // Primary action button
        Self::draw_button(f, primary_text, button_layout[3], false);
        
        // Stop button (disabled when idle/finished)
        let stop_disabled = matches!(app.phase, DefragPhase::Initializing | DefragPhase::Finished);
        Self::draw_button(f, "Stop", button_layout[5], stop_disabled);
    }
    
    /// Draw a Win98-style button with raised 3D effect
    fn draw_button(f: &mut Frame, text: &str, area: Rect, disabled: bool) {
        let (fg, bg) = if disabled {
            (colors::BUTTON_SHADOW, colors::BUTTON_FACE)
        } else {
            (colors::TEXT, colors::BUTTON_FACE)
        };
        
        // Win98 button style
        let button = Paragraph::new(text)
            .style(Style::default().fg(fg).bg(bg))
            .alignment(Alignment::Center);
        
        f.render_widget(button, area);
    }
}

// =============================================================================
// Custom Widgets
// =============================================================================

/// Win98-style disk grid widget that renders clusters as colored squares
struct Win98DiskGrid<'a> {
    clusters: &'a [ClusterState],
}

impl Widget for Win98DiskGrid<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid_width = area.width as usize;
        if grid_width == 0 {
            return;
        }
        
        // Grid layout: grid-cols-[repeat(auto-fit,minmax(8px,1fr))] gap-px
        // Each cluster is one character in terminal mode
        for (i, cluster) in self.clusters.iter().enumerate() {
            let x = (i % grid_width) as u16;
            let y = (i / grid_width) as u16;
            
            if y >= area.height {
                break;
            }
            
            // Convert to Win98 cluster state and get color
            let win98_state = Win98ClusterState::from(cluster);
            let color = win98_state.color();
            
            // Render as a solid block with the appropriate color
            if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                cell.set_symbol("█")
                    .set_fg(color)
                    .set_bg(Color::Black);
            }
        }
    }
}

/// Win98-style progress bar widget
struct Win98ProgressBar {
    progress: f64,  // 0.0 to 1.0
    fill_color: Color,
}

impl Widget for Win98ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 {
            return;
        }
        
        // Draw border (sunken effect) and progress fill
        let inner_width = area.width.saturating_sub(2);
        let filled_width = ((self.progress * inner_width as f64) as u16).min(inner_width);
        
        // Draw background (white) with borders
        for x in 0..area.width {
            if let Some(cell) = buf.cell_mut((area.x + x, area.y)) {
                if x == 0 {
                    // Left border (dark for sunken effect)
                    cell.set_symbol("▐").set_fg(colors::BUTTON_SHADOW).set_bg(Color::White);
                } else if x == area.width - 1 {
                    // Right border (light for sunken effect)
                    cell.set_symbol("▌").set_fg(colors::BUTTON_HIGHLIGHT).set_bg(Color::White);
                } else if x <= filled_width {
                    // Filled portion (navy blue)
                    cell.set_symbol("█").set_fg(self.fill_color).set_bg(Color::White);
                } else {
                    // Empty portion (white)
                    cell.set_symbol(" ").set_bg(Color::White);
                }
            }
        }
    }
}

// =============================================================================
// Win98 Render Function (called from main UI)
// =============================================================================

/// Main entry point to render the Win98 UI
pub fn render_win98_app(app: &App, frame: &mut Frame) {
    // Background: teal (like Win98 desktop)
    let bg_block = Block::default()
        .style(Style::default().bg(colors::DESKTOP_TEAL));
    frame.render_widget(bg_block, frame.area());
    
    // Center the window
    let area = frame.area();
    let window_width = area.width.min(80);
    let window_height = area.height.min(24);
    
    let window_x = (area.width.saturating_sub(window_width)) / 2;
    let window_y = (area.height.saturating_sub(window_height)) / 2;
    
    let window_area = Rect {
        x: window_x,
        y: window_y,
        width: window_width,
        height: window_height,
    };
    
    Win98Window::render(frame, app, window_area);
}
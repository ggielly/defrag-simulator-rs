//! Win98-style widgets for SDL2 rendering
//! Provides reusable UI components matching the Windows 98 look

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use super::sdl_backend::colors;

/// A rectangular area with position and size
#[derive(Debug, Clone, Copy)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Area {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn to_sdl_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
    
    pub fn inner(&self, margin: u32) -> Self {
        Self {
            x: self.x + margin as i32,
            y: self.y + margin as i32,
            width: self.width.saturating_sub(margin * 2),
            height: self.height.saturating_sub(margin * 2),
        }
    }
    
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }
}

/// Win98 Button states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

/// Win98-style Button widget
pub struct Button {
    pub area: Area,
    pub text: String,
    pub state: ButtonState,
    pub is_default: bool,
}

impl Button {
    pub fn new(x: i32, y: i32, width: u32, height: u32, text: &str) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            text: text.to_string(),
            state: ButtonState::Normal,
            is_default: false,
        }
    }
    
    pub fn with_default(mut self) -> Self {
        self.is_default = true;
        self
    }
    
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        let (x, y, w, h) = (self.area.x, self.area.y, self.area.width, self.area.height);
        
        // Fill background
        canvas.set_draw_color(colors::BUTTON_FACE);
        let _ = canvas.fill_rect(self.area.to_sdl_rect());
        
        match self.state {
            ButtonState::Pressed => {
                // Sunken border when pressed
                self.draw_sunken_border(canvas);
            }
            ButtonState::Disabled => {
                // Raised border but grayed out
                self.draw_raised_border(canvas);
            }
            _ => {
                // Normal raised border
                self.draw_raised_border(canvas);
                
                // Default button has extra black border
                if self.is_default {
                    canvas.set_draw_color(colors::BLACK);
                    let _ = canvas.draw_rect(Rect::new(x - 1, y - 1, w + 2, h + 2));
                }
            }
        }
    }
    
    fn draw_raised_border(&self, canvas: &mut Canvas<Window>) {
        let (x, y) = (self.area.x, self.area.y);
        let (w, h) = (self.area.width as i32, self.area.height as i32);
        
        // Outer highlight (top-left)
        canvas.set_draw_color(colors::BUTTON_HIGHLIGHT);
        let _ = canvas.draw_line((x, y), (x + w - 1, y));
        let _ = canvas.draw_line((x, y), (x, y + h - 1));
        
        // Outer shadow (bottom-right)
        canvas.set_draw_color(colors::WINDOW_FRAME);
        let _ = canvas.draw_line((x, y + h - 1), (x + w - 1, y + h - 1));
        let _ = canvas.draw_line((x + w - 1, y), (x + w - 1, y + h - 1));
        
        // Inner shadow
        canvas.set_draw_color(colors::BUTTON_SHADOW);
        let _ = canvas.draw_line((x + 1, y + h - 2), (x + w - 2, y + h - 2));
        let _ = canvas.draw_line((x + w - 2, y + 1), (x + w - 2, y + h - 2));
    }
    
    fn draw_sunken_border(&self, canvas: &mut Canvas<Window>) {
        let (x, y) = (self.area.x, self.area.y);
        let (w, h) = (self.area.width as i32, self.area.height as i32);
        
        // Outer shadow (top-left)
        canvas.set_draw_color(colors::BUTTON_SHADOW);
        let _ = canvas.draw_line((x, y), (x + w - 1, y));
        let _ = canvas.draw_line((x, y), (x, y + h - 1));
        
        // Outer highlight (bottom-right)  
        canvas.set_draw_color(colors::BUTTON_HIGHLIGHT);
        let _ = canvas.draw_line((x, y + h - 1), (x + w - 1, y + h - 1));
        let _ = canvas.draw_line((x + w - 1, y), (x + w - 1, y + h - 1));
    }
}

/// Win98-style Window widget
pub struct Win98WindowWidget {
    pub area: Area,
    pub title: String,
    pub active: bool,
    pub has_minimize: bool,
    pub has_maximize: bool,
    pub has_close: bool,
}

impl Win98WindowWidget {
    pub fn new(x: i32, y: i32, width: u32, height: u32, title: &str) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            title: title.to_string(),
            active: true,
            has_minimize: true,
            has_maximize: true,
            has_close: true,
        }
    }
    
    /// Get the title bar area
    pub fn title_bar_area(&self) -> Area {
        Area::new(self.area.x + 3, self.area.y + 3, self.area.width - 6, 18)
    }
    
    /// Get the client (content) area
    pub fn client_area(&self) -> Area {
        Area::new(
            self.area.x + 4,
            self.area.y + 25,
            self.area.width - 8,
            self.area.height - 29,
        )
    }
    
    /// Draw the window frame and title bar
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // Window background
        canvas.set_draw_color(colors::SURFACE);
        let _ = canvas.fill_rect(self.area.to_sdl_rect());
        
        // Window border (outer)
        self.draw_window_border(canvas);
        
        // Title bar
        self.draw_title_bar(canvas);
    }
    
    fn draw_window_border(&self, canvas: &mut Canvas<Window>) {
        let (x, y) = (self.area.x, self.area.y);
        let (w, h) = (self.area.width as i32, self.area.height as i32);
        
        // Outermost border
        canvas.set_draw_color(colors::BUTTON_FACE);
        let _ = canvas.draw_line((x, y), (x + w - 1, y));
        let _ = canvas.draw_line((x, y), (x, y + h - 1));
        
        canvas.set_draw_color(colors::WINDOW_FRAME);
        let _ = canvas.draw_line((x, y + h - 1), (x + w - 1, y + h - 1));
        let _ = canvas.draw_line((x + w - 1, y), (x + w - 1, y + h - 1));
        
        // Inner border (highlight)
        canvas.set_draw_color(colors::BUTTON_HIGHLIGHT);
        let _ = canvas.draw_line((x + 1, y + 1), (x + w - 2, y + 1));
        let _ = canvas.draw_line((x + 1, y + 1), (x + 1, y + h - 2));
        
        canvas.set_draw_color(colors::BUTTON_SHADOW);
        let _ = canvas.draw_line((x + 1, y + h - 2), (x + w - 2, y + h - 2));
        let _ = canvas.draw_line((x + w - 2, y + 1), (x + w - 2, y + h - 2));
    }
    
    fn draw_title_bar(&self, canvas: &mut Canvas<Window>) {
        let title_area = self.title_bar_area();
        
        // Title bar background (gradient simulation - we'll use solid color)
        let color = if self.active {
            colors::DIALOG_BLUE
        } else {
            colors::DIALOG_GRAY
        };
        
        canvas.set_draw_color(color);
        let _ = canvas.fill_rect(title_area.to_sdl_rect());
        
        // Draw title bar buttons
        self.draw_title_buttons(canvas);
    }
    
    fn draw_title_buttons(&self, canvas: &mut Canvas<Window>) {
        let title_area = self.title_bar_area();
        let btn_size = 14;
        let btn_y = title_area.y + 2;
        let mut btn_x = title_area.x + title_area.width as i32 - btn_size - 2;
        
        // Close button
        if self.has_close {
            self.draw_control_button(canvas, btn_x, btn_y, btn_size as u32, 'X');
            btn_x -= btn_size + 2;
        }
        
        // Maximize button
        if self.has_maximize {
            self.draw_control_button(canvas, btn_x, btn_y, btn_size as u32, '□');
            btn_x -= btn_size;
        }
        
        // Minimize button
        if self.has_minimize {
            self.draw_control_button(canvas, btn_x, btn_y, btn_size as u32, '_');
        }
    }
    
    fn draw_control_button(&self, canvas: &mut Canvas<Window>, x: i32, y: i32, size: u32, _icon: char) {
        // Button background
        canvas.set_draw_color(colors::BUTTON_FACE);
        let _ = canvas.fill_rect(Rect::new(x, y, size, size));
        
        // Raised border
        canvas.set_draw_color(colors::BUTTON_HIGHLIGHT);
        let _ = canvas.draw_line((x, y), (x + size as i32 - 1, y));
        let _ = canvas.draw_line((x, y), (x, y + size as i32 - 1));
        
        canvas.set_draw_color(colors::WINDOW_FRAME);
        let _ = canvas.draw_line((x, y + size as i32 - 1), (x + size as i32 - 1, y + size as i32 - 1));
        let _ = canvas.draw_line((x + size as i32 - 1, y), (x + size as i32 - 1, y + size as i32 - 1));
        
        canvas.set_draw_color(colors::BUTTON_SHADOW);
        let _ = canvas.draw_line((x + 1, y + size as i32 - 2), (x + size as i32 - 2, y + size as i32 - 2));
        let _ = canvas.draw_line((x + size as i32 - 2, y + 1), (x + size as i32 - 2, y + size as i32 - 2));
    }
}

/// Win98-style Progress Bar
pub struct ProgressBar {
    pub area: Area,
    pub progress: f64,  // 0.0 to 1.0
    pub fill_color: Color,
}

impl ProgressBar {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            progress: 0.0,
            fill_color: colors::DEFRAG_IDLE,
        }
    }
    
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.max(0.0).min(1.0);
    }
    
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // Background (white)
        canvas.set_draw_color(colors::WHITE);
        let _ = canvas.fill_rect(self.area.to_sdl_rect());
        
        // Sunken border
        self.draw_sunken_border(canvas);
        
        // Progress fill
        let inner = self.area.inner(2);
        let fill_width = ((inner.width as f64) * self.progress) as u32;
        if fill_width > 0 {
            canvas.set_draw_color(self.fill_color);
            let _ = canvas.fill_rect(Rect::new(inner.x, inner.y, fill_width, inner.height));
        }
    }
    
    fn draw_sunken_border(&self, canvas: &mut Canvas<Window>) {
        let (x, y) = (self.area.x, self.area.y);
        let (w, h) = (self.area.width as i32, self.area.height as i32);
        
        // Outer shadow (top-left)
        canvas.set_draw_color(colors::BUTTON_SHADOW);
        let _ = canvas.draw_line((x, y), (x + w - 1, y));
        let _ = canvas.draw_line((x, y), (x, y + h - 1));
        
        // Outer highlight (bottom-right)
        canvas.set_draw_color(colors::BUTTON_HIGHLIGHT);
        let _ = canvas.draw_line((x, y + h - 1), (x + w - 1, y + h - 1));
        let _ = canvas.draw_line((x + w - 1, y), (x + w - 1, y + h - 1));
    }
}

/// Win98-style Sunken Panel (for the disk grid)
pub struct SunkenPanel {
    pub area: Area,
    pub bg_color: Color,
}

impl SunkenPanel {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            bg_color: colors::BLACK,
        }
    }
    
    pub fn inner_area(&self) -> Area {
        self.area.inner(2)
    }
    
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        // Background
        canvas.set_draw_color(self.bg_color);
        let _ = canvas.fill_rect(self.area.to_sdl_rect());
        
        // Sunken border
        let (x, y) = (self.area.x, self.area.y);
        let (w, h) = (self.area.width as i32, self.area.height as i32);
        
        // Outer shadow (top-left)
        canvas.set_draw_color(colors::BUTTON_SHADOW);
        let _ = canvas.draw_line((x, y), (x + w - 1, y));
        let _ = canvas.draw_line((x, y), (x, y + h - 1));
        
        // Inner shadow
        canvas.set_draw_color(colors::WINDOW_FRAME);
        let _ = canvas.draw_line((x + 1, y + 1), (x + w - 2, y + 1));
        let _ = canvas.draw_line((x + 1, y + 1), (x + 1, y + h - 2));
        
        // Outer highlight (bottom-right)
        canvas.set_draw_color(colors::BUTTON_HIGHLIGHT);
        let _ = canvas.draw_line((x, y + h - 1), (x + w - 1, y + h - 1));
        let _ = canvas.draw_line((x + w - 1, y), (x + w - 1, y + h - 1));
        
        // Inner highlight
        canvas.set_draw_color(colors::BUTTON_FACE);
        let _ = canvas.draw_line((x + 1, y + h - 2), (x + w - 2, y + h - 2));
        let _ = canvas.draw_line((x + w - 2, y + 1), (x + w - 2, y + h - 2));
    }
}

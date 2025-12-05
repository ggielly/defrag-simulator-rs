//! SDL2 Backend for graphical rendering
//! Provides the core SDL2 initialization and event handling

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};

/// Windows 98 color palette (from CSS)
pub mod colors {
    use sdl2::pixels::Color;
    
    // Window chrome colors
    pub const SURFACE: Color = Color::RGB(192, 192, 192);           // silver
    pub const BUTTON_FACE: Color = Color::RGB(223, 223, 223);       // #dfdfdf
    pub const BUTTON_HIGHLIGHT: Color = Color::RGB(255, 255, 255);  // white
    pub const BUTTON_SHADOW: Color = Color::RGB(128, 128, 128);     // gray
    pub const WINDOW_FRAME: Color = Color::RGB(10, 10, 10);         // #0a0a0a
    
    // Title bar gradient colors
    pub const DIALOG_BLUE: Color = Color::RGB(0, 0, 128);           // navy
    pub const DIALOG_BLUE_LIGHT: Color = Color::RGB(16, 132, 208);  // #1084d0
    pub const DIALOG_GRAY: Color = Color::RGB(128, 128, 128);       // inactive
    
    // Defrag specific colors
    pub const DEFRAG_IDLE: Color = Color::RGB(0, 0, 128);           // navy (NOT_DEFRAGMENTED)
    pub const DEFRAG_PROGRESS: Color = Color::RGB(255, 0, 0);       // red (IN_PROGRESS)
    pub const DEFRAG_DONE: Color = Color::RGB(19, 250, 251);        // #13fafb (COMPLETED)
    
    // Text and background
    pub const TEXT: Color = Color::RGB(34, 34, 34);                 // #222
    pub const WHITE: Color = Color::RGB(255, 255, 255);
    pub const BLACK: Color = Color::RGB(0, 0, 0);
    pub const DESKTOP_TEAL: Color = Color::RGB(0, 128, 128);        // teal
}

/// Configuration for the SDL window
pub struct SdlConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub scale: u32,
}

impl Default for SdlConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            title: "Disk Defragmenter".to_string(),
            scale: 1,
        }
    }
}

/// SDL2 Backend managing the window and rendering context
pub struct SdlBackend {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>,
    pub ttf_context: Sdl2TtfContext,
    pub event_pump: sdl2::EventPump,
    pub config: SdlConfig,
    pub running: bool,
}

impl SdlBackend {
    /// Create a new SDL2 backend with the given configuration
    pub fn new(config: SdlConfig) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        
        let window = video_subsystem
            .window(&config.title, config.width * config.scale, config.height * config.scale)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;
        
        let mut canvas = window.into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        
        // Set logical size for pixel-perfect scaling
        canvas.set_logical_size(config.width, config.height)
            .map_err(|e| e.to_string())?;
        
        let texture_creator = canvas.texture_creator();
        let event_pump = sdl_context.event_pump()?;
        
        Ok(Self {
            sdl_context,
            video_subsystem,
            canvas,
            texture_creator,
            ttf_context,
            event_pump,
            config,
            running: true,
        })
    }
    
    /// Clear the canvas with the desktop color
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(colors::DESKTOP_TEAL);
        self.canvas.clear();
    }
    
    /// Present the canvas to the screen
    pub fn present(&mut self) {
        self.canvas.present();
    }
    
    /// Poll events and return true if the application should continue running
    pub fn poll_events(&mut self) -> Vec<SdlEvent> {
        let mut events = Vec::new();
        
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.running = false;
                    events.push(SdlEvent::Quit);
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    events.push(SdlEvent::KeyDown(keycode));
                    if keycode == Keycode::Escape {
                        self.running = false;
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    events.push(SdlEvent::KeyUp(keycode));
                }
                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    events.push(SdlEvent::MouseDown { x, y, button: mouse_btn });
                }
                Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                    events.push(SdlEvent::MouseUp { x, y, button: mouse_btn });
                }
                Event::MouseMotion { x, y, .. } => {
                    events.push(SdlEvent::MouseMove { x, y });
                }
                _ => {}
            }
        }
        
        events
    }
    
    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.fill_rect(Rect::new(x, y, w, h));
    }
    
    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.draw_rect(Rect::new(x, y, w, h));
    }
    
    /// Draw a horizontal line
    pub fn draw_hline(&mut self, x1: i32, x2: i32, y: i32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.draw_line((x1, y), (x2, y));
    }
    
    /// Draw a vertical line
    pub fn draw_vline(&mut self, x: i32, y1: i32, y2: i32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.draw_line((x, y1), (x, y2));
    }
    
    /// Draw a Win98-style raised border (3D effect)
    pub fn draw_raised_border(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let w = w as i32;
        let h = h as i32;
        
        // Outer highlight (top-left)
        self.draw_hline(x, x + w - 1, y, colors::BUTTON_HIGHLIGHT);
        self.draw_vline(x, y, y + h - 1, colors::BUTTON_HIGHLIGHT);
        
        // Inner highlight
        self.draw_hline(x + 1, x + w - 2, y + 1, colors::BUTTON_FACE);
        self.draw_vline(x + 1, y + 1, y + h - 2, colors::BUTTON_FACE);
        
        // Outer shadow (bottom-right)
        self.draw_hline(x, x + w - 1, y + h - 1, colors::WINDOW_FRAME);
        self.draw_vline(x + w - 1, y, y + h - 1, colors::WINDOW_FRAME);
        
        // Inner shadow
        self.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_SHADOW);
        self.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_SHADOW);
    }
    
    /// Draw a Win98-style sunken border (3D effect for panels)
    pub fn draw_sunken_border(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let w = w as i32;
        let h = h as i32;
        
        // Outer shadow (top-left)
        self.draw_hline(x, x + w - 1, y, colors::BUTTON_SHADOW);
        self.draw_vline(x, y, y + h - 1, colors::BUTTON_SHADOW);
        
        // Inner shadow
        self.draw_hline(x + 1, x + w - 2, y + 1, colors::WINDOW_FRAME);
        self.draw_vline(x + 1, y + 1, y + h - 2, colors::WINDOW_FRAME);
        
        // Outer highlight (bottom-right)
        self.draw_hline(x, x + w - 1, y + h - 1, colors::BUTTON_HIGHLIGHT);
        self.draw_vline(x + w - 1, y, y + h - 1, colors::BUTTON_HIGHLIGHT);
        
        // Inner highlight
        self.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_FACE);
        self.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_FACE);
    }
    
    /// Draw a Win98-style window border
    pub fn draw_window_border(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let w = w as i32;
        let h = h as i32;
        
        // Outer border
        self.draw_hline(x, x + w - 1, y, colors::BUTTON_FACE);
        self.draw_vline(x, y, y + h - 1, colors::BUTTON_FACE);
        self.draw_hline(x, x + w - 1, y + h - 1, colors::WINDOW_FRAME);
        self.draw_vline(x + w - 1, y, y + h - 1, colors::WINDOW_FRAME);
        
        // Inner border
        self.draw_hline(x + 1, x + w - 2, y + 1, colors::BUTTON_HIGHLIGHT);
        self.draw_vline(x + 1, y + 1, y + h - 2, colors::BUTTON_HIGHLIGHT);
        self.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_SHADOW);
        self.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_SHADOW);
    }
    
    /// Draw text at the given position
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, size: u16, color: Color) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, 0));
        }
        
        let font = self.ttf_context
            .load_font_from_rwops(
                sdl2::rwops::RWops::from_bytes(super::fonts::FONT_DATA)
                    .map_err(|e| format!("Failed to create RWops: {}", e))?,
                size,
            )
            .map_err(|e| format!("Failed to load font: {}", e))?;
        
        let surface = font
            .render(text)
            .blended(color)
            .map_err(|e| format!("Failed to render text: {}", e))?;
        
        let texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture: {}", e))?;
        
        let sdl2::render::TextureQuery { width, height, .. } = texture.query();
        
        let target = Rect::new(x, y, width, height);
        self.canvas.copy(&texture, None, Some(target))
            .map_err(|e| format!("Failed to copy texture: {}", e))?;
        
        Ok((width, height))
    }
    
    /// Draw text centered within a given width
    pub fn draw_text_centered(&mut self, text: &str, x: i32, y: i32, width: u32, size: u16, color: Color) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, 0));
        }
        
        let font = self.ttf_context
            .load_font_from_rwops(
                sdl2::rwops::RWops::from_bytes(super::fonts::FONT_DATA)
                    .map_err(|e| format!("Failed to create RWops: {}", e))?,
                size,
            )
            .map_err(|e| format!("Failed to load font: {}", e))?;
        
        let (text_width, _) = font.size_of(text)
            .map_err(|e| format!("Failed to measure text: {}", e))?;
        
        let centered_x = x + ((width as i32 - text_width as i32) / 2);
        
        self.draw_text(text, centered_x, y, size, color)
    }
    
    /// Check if still running
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    /// Get window dimensions
    pub fn get_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

/// Simplified SDL event types
#[derive(Debug, Clone)]
pub enum SdlEvent {
    Quit,
    KeyDown(Keycode),
    KeyUp(Keycode),
    MouseDown { x: i32, y: i32, button: sdl2::mouse::MouseButton },
    MouseUp { x: i32, y: i32, button: sdl2::mouse::MouseButton },
    MouseMove { x: i32, y: i32 },
}

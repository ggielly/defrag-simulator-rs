//! SDL2 Backend for graphical rendering
//! Provides the core SDL2 initialization, event handling, and font caching.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;

/// Windows 98 color palette.
pub mod colors {
    use sdl2::pixels::Color;

    pub const SURFACE: Color = Color::RGB(192, 192, 192);
    pub const BUTTON_FACE: Color = Color::RGB(223, 223, 223);
    pub const BUTTON_HIGHLIGHT: Color = Color::RGB(255, 255, 255);
    pub const BUTTON_SHADOW: Color = Color::RGB(128, 128, 128);
    pub const WINDOW_FRAME: Color = Color::RGB(10, 10, 10);

    pub const DIALOG_BLUE: Color = Color::RGB(0, 0, 128);
    pub const DIALOG_BLUE_LIGHT: Color = Color::RGB(16, 132, 208);
    pub const DIALOG_GRAY: Color = Color::RGB(128, 128, 128);

    pub const DEFRAG_IDLE: Color = Color::RGB(0, 0, 128);
    pub const DEFRAG_PROGRESS: Color = Color::RGB(255, 0, 0);
    pub const DEFRAG_DONE: Color = Color::RGB(19, 250, 251);

    pub const TEXT: Color = Color::RGB(34, 34, 34);
    pub const WHITE: Color = Color::RGB(255, 255, 255);
    pub const BLACK: Color = Color::RGB(0, 0, 0);
    pub const DESKTOP_TEAL: Color = Color::RGB(0, 128, 128);

    pub const DISABLED_TEXT: Color = Color::RGB(160, 160, 160);
}

/// Configuration for the SDL window.
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

/// SDL2 Backend managing the window, rendering context, and cached fonts.
pub struct SdlBackend {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>,
    pub ttf_context: Sdl2TtfContext,
    pub event_pump: sdl2::EventPump,
    pub config: SdlConfig,
    pub running: bool,
    fonts: HashMap<u16, Font<'static, 'static>>,
}

impl SdlBackend {
    /// Create a new SDL2 backend with the given configuration.
    pub fn new(config: SdlConfig) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        let window = video_subsystem
            .window(
                &config.title,
                config.width * config.scale,
                config.height * config.scale,
            )
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        canvas
            .set_logical_size(config.width, config.height)
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
            fonts: HashMap::new(),
        })
    }

    fn get_font(&mut self, size: u16) -> Result<&Font<'static, 'static>, String> {
        if !self.fonts.contains_key(&size) {
            let font = self
                .ttf_context
                .load_font_from_rwops(
                    sdl2::rwops::RWops::from_bytes(super::fonts::FONT_DATA)
                        .map_err(|e| format!("Failed to create RWops: {}", e))?,
                    size,
                )
                .map_err(|e| format!("Failed to load font: {}", e))?;
            self.fonts.insert(size, font);
        }
        Ok(self.fonts.get(&size).unwrap())
    }

    // -- Drawing primitives ---------------------------------------------------

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(colors::DESKTOP_TEAL);
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.fill_rect(Rect::new(x, y, w, h));
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.draw_rect(Rect::new(x, y, w, h));
    }

    pub fn draw_hline(&mut self, x1: i32, x2: i32, y: i32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.draw_line((x1, y), (x2, y));
    }

    pub fn draw_vline(&mut self, x: i32, y1: i32, y2: i32, color: Color) {
        self.canvas.set_draw_color(color);
        let _ = self.canvas.draw_line((x, y1), (x, y2));
    }

    pub fn draw_raised_border(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let w = w as i32;
        let h = h as i32;
        self.draw_hline(x, x + w - 1, y, colors::BUTTON_HIGHLIGHT);
        self.draw_vline(x, y, y + h - 1, colors::BUTTON_HIGHLIGHT);
        self.draw_hline(x + 1, x + w - 2, y + 1, colors::BUTTON_FACE);
        self.draw_vline(x + 1, y + 1, y + h - 2, colors::BUTTON_FACE);
        self.draw_hline(x, x + w - 1, y + h - 1, colors::WINDOW_FRAME);
        self.draw_vline(x + w - 1, y, y + h - 1, colors::WINDOW_FRAME);
        self.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_SHADOW);
        self.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_SHADOW);
    }

    pub fn draw_sunken_border(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let w = w as i32;
        let h = h as i32;
        self.draw_hline(x, x + w - 1, y, colors::BUTTON_SHADOW);
        self.draw_vline(x, y, y + h - 1, colors::BUTTON_SHADOW);
        self.draw_hline(x + 1, x + w - 2, y + 1, colors::WINDOW_FRAME);
        self.draw_vline(x + 1, y + 1, y + h - 2, colors::WINDOW_FRAME);
        self.draw_hline(x, x + w - 1, y + h - 1, colors::BUTTON_HIGHLIGHT);
        self.draw_vline(x + w - 1, y, y + h - 1, colors::BUTTON_HIGHLIGHT);
        self.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_FACE);
        self.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_FACE);
    }

    // -- Text -----------------------------------------------------------------

    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        size: u16,
        color: Color,
    ) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, 0));
        }
        let font = self.get_font(size)?;
        let surface = font
            .render(text)
            .blended(color)
            .map_err(|e| format!("Failed to render text: {}", e))?;
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture: {}", e))?;
        let q = texture.query();
        let target = Rect::new(x, y, q.width, q.height);
        self.canvas
            .copy(&texture, None, Some(target))
            .map_err(|e| format!("Failed to copy texture: {}", e))?;
        Ok((q.width, q.height))
    }

    pub fn draw_text_centered(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        width: u32,
        size: u16,
        color: Color,
    ) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, 0));
        }
        let (tw, _) = self.get_text_width(text, size)?;
        let cx = x + ((width as i32 - tw as i32) / 2);
        self.draw_text(text, cx, y, size, color)
    }

    pub fn get_text_width(&mut self, text: &str, size: u16) -> Result<u32, String> {
        if text.is_empty() {
            return Ok(0);
        }
        let font = self.get_font(size)?;
        let (w, _) = font
            .size_of(text)
            .map_err(|e| format!("Failed to measure text: {}", e))?;
        Ok(w)
    }

    // -- Events ---------------------------------------------------------------

    pub fn poll_events(&mut self) -> Vec<SdlEvent> {
        let mut events = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.running = false;
                    events.push(SdlEvent::Quit);
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    events.push(SdlEvent::KeyDown(keycode));
                    if keycode == Keycode::Escape {
                        self.running = false;
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    events.push(SdlEvent::KeyUp(keycode));
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    events.push(SdlEvent::MouseDown { x, y, button: mouse_btn });
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
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

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

/// Simplified SDL event types.
#[derive(Debug, Clone)]
pub enum SdlEvent {
    Quit,
    KeyDown(Keycode),
    KeyUp(Keycode),
    MouseDown {
        x: i32,
        y: i32,
        button: sdl2::mouse::MouseButton,
    },
    MouseUp {
        x: i32,
        y: i32,
        button: sdl2::mouse::MouseButton,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
}

//! Font management for SDL2 graphical rendering
//! Handles loading and rendering text with Win98-style fonts

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureQuery};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::Window;
use std::path::Path;

/// Embedded font data (VT323 - a pixel-style font)
pub const FONT_DATA: &[u8] = include_bytes!("../../static/fonts/VT323.ttf");

/// Font sizes used in Win98 UI
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSize {
    Small = 11,
    Normal = 13,
    Large = 16,
    Title = 14,
}

/// Font manager for the Win98 UI
pub struct FontManager<'ttf> {
    pub font_small: Font<'ttf, 'static>,
    pub font_normal: Font<'ttf, 'static>,
    pub font_large: Font<'ttf, 'static>,
    pub font_title: Font<'ttf, 'static>,
}

impl<'ttf> FontManager<'ttf> {
    /// Load fonts from embedded data
    pub fn new(ttf_context: &'ttf Sdl2TtfContext) -> Result<Self, String> {
        // Load from embedded font data
        let rwops_small = sdl2::rwops::RWops::from_bytes(FONT_DATA)
            .map_err(|e| format!("Failed to create RWops: {}", e))?;
        let rwops_normal = sdl2::rwops::RWops::from_bytes(FONT_DATA)
            .map_err(|e| format!("Failed to create RWops: {}", e))?;
        let rwops_large = sdl2::rwops::RWops::from_bytes(FONT_DATA)
            .map_err(|e| format!("Failed to create RWops: {}", e))?;
        let rwops_title = sdl2::rwops::RWops::from_bytes(FONT_DATA)
            .map_err(|e| format!("Failed to create RWops: {}", e))?;

        let font_small = ttf_context
            .load_font_from_rwops(rwops_small, FontSize::Small as u16)
            .map_err(|e| format!("Failed to load small font: {}", e))?;
        let font_normal = ttf_context
            .load_font_from_rwops(rwops_normal, FontSize::Normal as u16)
            .map_err(|e| format!("Failed to load normal font: {}", e))?;
        let font_large = ttf_context
            .load_font_from_rwops(rwops_large, FontSize::Large as u16)
            .map_err(|e| format!("Failed to load large font: {}", e))?;
        let font_title = ttf_context
            .load_font_from_rwops(rwops_title, FontSize::Title as u16)
            .map_err(|e| format!("Failed to load title font: {}", e))?;

        Ok(Self {
            font_small,
            font_normal,
            font_large,
            font_title,
        })
    }

    /// Load fonts from file path (alternative to embedded)
    pub fn from_file(ttf_context: &'ttf Sdl2TtfContext, font_path: &Path) -> Result<Self, String> {
        let font_small = ttf_context
            .load_font(font_path, FontSize::Small as u16)
            .map_err(|e| format!("Failed to load font: {}", e))?;
        let font_normal = ttf_context
            .load_font(font_path, FontSize::Normal as u16)
            .map_err(|e| format!("Failed to load font: {}", e))?;
        let font_large = ttf_context
            .load_font(font_path, FontSize::Large as u16)
            .map_err(|e| format!("Failed to load font: {}", e))?;
        let font_title = ttf_context
            .load_font(font_path, FontSize::Title as u16)
            .map_err(|e| format!("Failed to load font: {}", e))?;

        Ok(Self {
            font_small,
            font_normal,
            font_large,
            font_title,
        })
    }

    /// Get font by size
    pub fn get_font(&self, size: FontSize) -> &Font<'ttf, 'static> {
        match size {
            FontSize::Small => &self.font_small,
            FontSize::Normal => &self.font_normal,
            FontSize::Large => &self.font_large,
            FontSize::Title => &self.font_title,
        }
    }
}

/// Text rendering helper
pub struct TextRenderer;

impl TextRenderer {
    /// Render text to the canvas at the given position
    pub fn draw_text<'a>(
        canvas: &mut Canvas<Window>,
        font: &Font<'_, '_>,
        text: &str,
        x: i32,
        y: i32,
        color: Color,
    ) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, 0));
        }

        let texture_creator = canvas.texture_creator();

        let surface = font
            .render(text)
            .blended(color)
            .map_err(|e| format!("Failed to render text: {}", e))?;

        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| format!("Failed to create texture: {}", e))?;

        let TextureQuery { width, height, .. } = texture.query();

        let target = Rect::new(x, y, width, height);
        canvas
            .copy(&texture, None, Some(target))
            .map_err(|e| format!("Failed to copy texture: {}", e))?;

        Ok((width, height))
    }

    /// Render text centered horizontally within a given width
    pub fn draw_text_centered<'a>(
        canvas: &mut Canvas<Window>,
        font: &Font<'_, '_>,
        text: &str,
        x: i32,
        y: i32,
        width: u32,
        color: Color,
    ) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, 0));
        }

        // Calculate text width first
        let (text_width, _) = font
            .size_of(text)
            .map_err(|e| format!("Failed to measure text: {}", e))?;

        let centered_x = x + ((width as i32 - text_width as i32) / 2);

        Self::draw_text(canvas, font, text, centered_x, y, color)
    }

    /// Render text with a shadow (Win98 style for title bars)
    pub fn draw_text_shadowed<'a>(
        canvas: &mut Canvas<Window>,
        font: &Font<'_, '_>,
        text: &str,
        x: i32,
        y: i32,
        color: Color,
        shadow_color: Color,
    ) -> Result<(u32, u32), String> {
        // Draw shadow first (offset by 1,1)
        Self::draw_text(canvas, font, text, x + 1, y + 1, shadow_color)?;
        // Draw main text
        Self::draw_text(canvas, font, text, x, y, color)
    }

    /// Measure text dimensions without rendering
    pub fn measure_text(font: &Font<'_, '_>, text: &str) -> Result<(u32, u32), String> {
        if text.is_empty() {
            return Ok((0, font.height() as u32));
        }
        font.size_of(text)
            .map_err(|e| format!("Failed to measure text: {}", e))
    }
}

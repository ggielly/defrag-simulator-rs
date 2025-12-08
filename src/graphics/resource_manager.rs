//! Resource Manager for handling textures and other graphical resources
//! Provides a centralized way to load, cache and manage graphical assets
//! Designed for reuse across different UIs (Win95, Win98, Symantec defrag, etc.)

use image::RgbaImage;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::Window;
use std::collections::HashMap;
use std::path::Path;

/// Type alias for texture IDs
pub type TextureId = String;

/// Result type for resource manager operations
pub type ResourceManagerResult<T> = Result<T, ResourceManagerError>;

/// Error types for the resource manager
#[derive(Debug)]
pub enum ResourceManagerError {
    ImageError(String),
    TextureCreationError(String),
    MissingResource(String),
}

impl std::fmt::Display for ResourceManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceManagerError::ImageError(msg) => write!(f, "Image error: {}", msg),
            ResourceManagerError::TextureCreationError(msg) => {
                write!(f, "Texture creation error: {}", msg)
            }
            ResourceManagerError::MissingResource(name) => write!(f, "Missing resource: {}", name),
        }
    }
}

impl std::error::Error for ResourceManagerError {}

/// A resource that contains pre-created textures
struct CachedTexture {
    image: RgbaImage,
    textures: HashMap<String, Texture<'static>>, // We cache texture handles
}

/// Resource cache for storing loaded images and textures
pub struct ResourceCache {
    images: HashMap<TextureId, RgbaImage>,
}

impl ResourceCache {
    /// Creates a new resource cache
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    /// Loads an image from file and stores it in the cache
    pub fn load_image_from_file<P: AsRef<Path>>(
        &mut self,
        id: &str,
        path: P,
    ) -> ResourceManagerResult<()> {
        let img = image::open(path)
            .map(|img| img.to_rgba8())
            .map_err(|e| ResourceManagerError::ImageError(e.to_string()))?;

        self.images.insert(id.to_string(), img);
        Ok(())
    }

    /// Loads an image from bytes and stores it in the cache
    pub fn load_image_from_bytes(&mut self, id: &str, data: &[u8]) -> ResourceManagerResult<()> {
        let img = image::load_from_memory(data)
            .map(|img| img.to_rgba8())
            .map_err(|e| ResourceManagerError::ImageError(e.to_string()))?;

        self.images.insert(id.to_string(), img);
        Ok(())
    }

    /// Gets a reference to a cached image
    pub fn get_image(&self, id: &str) -> ResourceManagerResult<&RgbaImage> {
        self.images
            .get(id)
            .ok_or_else(|| ResourceManagerError::MissingResource(id.to_string()))
    }

    /// Checks if an image exists in the cache
    pub fn has_image(&self, id: &str) -> bool {
        self.images.contains_key(id)
    }

    /// Creates a texture from a cached image using the TextureCreator
    /// This approach avoids the borrowing issue by using the TextureCreator separately
    pub fn create_texture_from_cached_image(
        &self,
        texture_creator: &TextureCreator<Window>,
        id: &str,
    ) -> ResourceManagerResult<Texture> {
        let img = self.get_image(id)?;
        let (width, height) = img.dimensions();

        let mut texture = texture_creator
            .create_texture_target(sdl2::pixels::PixelFormatEnum::RGBA8888, width, height)
            .map_err(|e| ResourceManagerError::TextureCreationError(e.to_string()))?;

        // Update the texture with image data
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                // Copy image data to buffer
                for (_i, (x, y, pixel)) in img.enumerate_pixels().enumerate() {
                    let buffer_index = y as usize * pitch + x as usize * 4;
                    if buffer_index + 3 < buffer.len() {
                        buffer[buffer_index] = pixel[0]; // R
                        buffer[buffer_index + 1] = pixel[1]; // G
                        buffer[buffer_index + 2] = pixel[2]; // B
                        buffer[buffer_index + 3] = pixel[3]; // A
                    }
                }
            })
            .map_err(|e| ResourceManagerError::TextureCreationError(e.to_string()))?;

        Ok(texture)
    }

    /// Checks if cache is empty
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Clears all cached images
    pub fn clear(&mut self) {
        self.images.clear();
    }
}

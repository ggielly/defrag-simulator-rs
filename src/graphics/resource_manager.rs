//! Resource Manager for handling textures and other graphical resources
//! Provides a centralized way to load, cache and manage graphical assets
//! Designed for reuse across different UIs (Win95, Win98, Symantec defrag, etc.)

use image::RgbaImage;
use std::collections::HashMap;
use std::path::Path;

/// Result type for resource manager operations
pub type ResourceManagerResult<T> = Result<T, ResourceManagerError>;

/// Error types for the resource manager
#[derive(Debug)]
pub enum ResourceManagerError {
    ImageError(String),
    MissingResource(String),
}

impl std::fmt::Display for ResourceManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceManagerError::ImageError(msg) => write!(f, "Image error: {}", msg),
            ResourceManagerError::MissingResource(name) => write!(f, "Missing resource: {}", name),
        }
    }
}

impl std::error::Error for ResourceManagerError {}

/// Resource cache for storing loaded images
pub struct ResourceCache {
    images: HashMap<String, RgbaImage>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

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

    pub fn load_image_from_bytes(&mut self, id: &str, data: &[u8]) -> ResourceManagerResult<()> {
        let img = image::load_from_memory(data)
            .map(|img| img.to_rgba8())
            .map_err(|e| ResourceManagerError::ImageError(e.to_string()))?;
        self.images.insert(id.to_string(), img);
        Ok(())
    }

    pub fn get_image(&self, id: &str) -> ResourceManagerResult<&RgbaImage> {
        self.images
            .get(id)
            .ok_or_else(|| ResourceManagerError::MissingResource(id.to_string()))
    }

    pub fn has_image(&self, id: &str) -> bool {
        self.images.contains_key(id)
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn clear(&mut self) {
        self.images.clear();
    }
}

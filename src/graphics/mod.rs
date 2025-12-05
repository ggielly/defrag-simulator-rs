//! Graphical rendering module for Windows 95/98 style interfaces
//! Uses SDL2 for pixel-perfect recreation of the classic Windows look

#[cfg(feature = "graphical")]
pub mod sdl_backend;

#[cfg(feature = "graphical")]
pub mod win98_renderer;

#[cfg(feature = "graphical")]
pub mod win98_widgets;

#[cfg(feature = "graphical")]
pub mod fonts;

#[cfg(feature = "graphical")]
pub use sdl_backend::SdlBackend;

#[cfg(feature = "graphical")]
pub use win98_renderer::Win98GraphicalRenderer;

#[cfg(feature = "graphical")]
pub use fonts::{FontManager, FontSize, TextRenderer};

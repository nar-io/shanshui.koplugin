//! Brush primitives built on the display list: strokes, blobs, texture.

pub mod blob;
pub mod stroke;
pub mod texture;

pub use blob::{BlobStyle, blob, blob_outline};
pub use stroke::{StrokeStyle, stroke};
pub use texture::{TextureStyle, texture};

/// Constant width profile, the `function(x){return 1}` of the original.
pub fn flat_profile(_t: f64) -> f64 {
    1.0
}

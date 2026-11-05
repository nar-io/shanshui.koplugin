//! What a polygon is filled or outlined with. Every colour in the original is
//! a grey `rgba(g,g,g,a)`, `white` (paper that hides what lies behind), or
//! `none`.

/// Fill or outline paint of an [`super::InkPolygon`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    /// Nothing drawn (`none` / `rgba(0,0,0,0)`).
    Clear,
    /// Opaque paper: erases ink under it (the original's `white` fills).
    Paper,
    /// Grey ink composited with `alpha`.
    Ink { gray: u8, alpha: f32 },
}

impl Paint {
    /// `rgba(gray,gray,gray,alpha)`; maps terminal gray levels back to authentic Chinese ink tones (25 for pure ink).
    pub fn ink(gray: u8, alpha: f64) -> Self {
        let actual_gray = if gray <= 100 {
            25
        } else if gray <= 140 {
            30
        } else {
            // Keep lighter tones proportional (clouds, mist, distant mountains)
            (((gray as f32 - 100.0) / 155.0) * 160.0 + 35.0).round() as u8
        };
        Self::Ink {
            gray: actual_gray,
            alpha: alpha.clamp(0.0, 1.0) as f32,
        }
    }

    /// Same pigment, different opacity.
    pub fn with_alpha(self, alpha: f64) -> Self {
        let alpha = alpha.clamp(0.0, 1.0) as f32;
        match self {
            Self::Ink { gray, .. } => Self::Ink { gray, alpha },
            other => other,
        }
    }

    /// Opacity, 1 for paper and 0 for clear.
    pub fn alpha(self) -> f64 {
        match self {
            Self::Clear => 0.0,
            Self::Paper => 1.0,
            Self::Ink { alpha, .. } => f64::from(alpha),
        }
    }

    pub fn is_visible(self) -> bool {
        self.alpha() > 0.0
    }
}

//! Which part of the world is on screen and at what scale (the
//! `calcViewBox` of the original, adapted to a terminal-sized dot grid).

use crate::geom::Pt;

/// Height of the original view box: `MEM.windy / zoom = 800 / 1.142`.
pub const WORLD_HEIGHT: f64 = 700.0;

/// World rectangle mapped onto a `dots_w × dots_h` grid of square dots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// World x of the left edge (`MEM.cursx`).
    pub left: f64,
    /// World y of the top edge.
    pub top: f64,
    /// World units per dot.
    pub scale: f64,
    pub dots_w: usize,
    pub dots_h: usize,
}

impl Viewport {
    /// Fits `WORLD_HEIGHT / zoom` world units into `dots_h` dots; `pan`
    /// in `[0, 1]` picks which slice when zoomed in (1 = bottom).
    /// Example: `Viewport::fit(0.0, 1.0, 1.0, 400, 200)`.
    pub fn fit(left: f64, zoom: f64, pan: f64, dots_w: usize, dots_h: usize) -> Self {
        let visible = WORLD_HEIGHT / zoom.max(1.0);
        let top = (WORLD_HEIGHT - visible) * pan.clamp(0.0, 1.0);
        let scale = visible / dots_h.max(1) as f64;
        Self {
            left,
            top,
            scale,
            dots_w,
            dots_h,
        }
    }

    pub fn world_width(&self) -> f64 {
        self.dots_w as f64 * self.scale
    }

    /// `[xmin, ymin, xmax, ymax]` in world units.
    pub fn world_rect(&self) -> [f64; 4] {
        let h = self.dots_h as f64 * self.scale;
        [
            self.left,
            self.top,
            self.left + self.world_width(),
            self.top + h,
        ]
    }

    /// World point → fractional dot coordinates.
    pub fn to_dots(&self, p: Pt) -> Pt {
        [
            (p[0] - self.left) / self.scale,
            (p[1] - self.top) / self.scale,
        ]
    }

    /// Whether a world bbox overlaps the view.
    pub fn sees(&self, bbox: [f64; 4]) -> bool {
        let r = self.world_rect();
        bbox[2] >= r[0] && bbox[0] <= r[2] && bbox[3] >= r[1] && bbox[1] <= r[3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_full_height() {
        let v = Viewport::fit(100.0, 1.0, 1.0, 400, 200);
        assert_eq!(v.scale, 3.5);
        assert_eq!(v.world_width(), 1400.0);
        assert_eq!(v.to_dots([100.0, 700.0]), [0.0, 200.0]);
    }

    #[test]
    fn zoom_anchors_by_pan() {
        let bottom = Viewport::fit(0.0, 2.0, 1.0, 10, 10);
        let top = Viewport::fit(0.0, 2.0, 0.0, 10, 10);
        assert_eq!(bottom.top, 350.0);
        assert_eq!(top.top, 0.0);
    }

    #[test]
    fn sees_overlap_only() {
        let v = Viewport::fit(0.0, 1.0, 1.0, 100, 100);
        assert!(v.sees([-10.0, 0.0, 1.0, 10.0]));
        assert!(!v.sees([800.0, 0.0, 900.0, 10.0]));
    }
}

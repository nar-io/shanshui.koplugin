//! `Mount.distMount`: a far-away range, painted as flat grey facets with
//! no outlines — atmospheric perspective.

use std::f64::consts::PI;

use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::mid_pt;
use crate::geom::triangulate::triangulate;
use crate::ink::{Paint, PolyStyle, Sketch};

const SPAN: f64 = 10.0;
const SEG: usize = 5;

/// Example: `dist_mount(&mut sk, &mut ch, x, 260.0, 42.0, 150.0, 1000.0)`.
pub fn dist_mount(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    seed: f64,
    hei: f64,
    len: f64,
) {
    let range = Range {
        xoff,
        yoff,
        seed,
        hei,
        steps: len / SPAN,
    };
    let strips = (range.steps / SEG as f64).ceil() as usize;
    for i in 0..strips {
        let strip = range.strip(chance, i);
        let base = range.ink(chance, strip[strip.len() - 1]);
        sketch.poly(strip.clone(), PolyStyle::filled(base));
        for tri in triangulate(&strip, 100.0, false) {
            let ink = range.ink(chance, mid_pt(&tri));
            sketch.poly(tri.to_vec(), PolyStyle::solid(ink, 1.0));
        }
    }
}

struct Range {
    xoff: f64,
    yoff: f64,
    seed: f64,
    hei: f64,
    steps: f64,
}

impl Range {
    /// Ridge points of strip `i`, preceded (reversed) by its base points.
    fn strip(&self, chance: &Chance, i: usize) -> Vec<Pt> {
        let base: Vec<Pt> = (0..=SEG / 2 + 1)
            .map(|j| self.base_point(chance, i * SEG + j * 2))
            .collect();
        let ridge = (0..=SEG).map(|j| self.ridge_point(chance, i * SEG + j));
        base.into_iter().rev().chain(ridge).collect()
    }

    fn envelope(&self, k: f64) -> f64 {
        (PI * k / self.steps).sin()
    }

    fn ridge_point(&self, chance: &Chance, k: usize) -> Pt {
        let k = k as f64;
        let h = self.hei * chance.noise2(k * 0.05, self.seed) * self.envelope(k).max(0.0).sqrt();
        [self.xoff + k * SPAN, self.yoff - h]
    }

    fn base_point(&self, chance: &Chance, k: usize) -> Pt {
        let k = k as f64;
        let h = 24.0 * chance.noise(k * 0.05, 2.0, self.seed) * self.envelope(k);
        [self.xoff + k * SPAN, self.yoff + h]
    }

    /// Opaque light grey varying with position.
    fn ink(&self, chance: &Chance, p: Pt) -> Paint {
        Paint::ink(
            (chance.noise(p[0] * 0.02, p[1] * 0.02, self.yoff) * 55.0 + 200.0) as u8,
            1.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_is_light_and_opaque() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        dist_mount(&mut s, &mut c, 0.0, 280.0, 3.0, 150.0, 500.0);
        assert!(s.len() > 10);
        for p in &s.polygons {
            match p.style.fill {
                Paint::Ink { gray, alpha } => assert!(gray >= 200 && alpha == 1.0),
                other => panic!("unexpected fill {other:?}"),
            }
        }
    }

    #[test]
    fn strip_has_base_and_ridge() {
        let c = Chance::from_seed(2);
        let r = Range {
            xoff: 0.0,
            yoff: 0.0,
            seed: 0.0,
            hei: 100.0,
            steps: 50.0,
        };
        assert_eq!(r.strip(&c, 0).len(), (SEG / 2 + 2) + (SEG + 1));
    }
}

//! Hats (`hat01`, `hat02`) and carried items (`stick01`), drawn in the
//! frame of the neck→head bone.

use std::f64::consts::PI;

use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::distance;
use crate::ink::{Paint, PolyStyle, Sketch};

/// Which hat a person wears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hat {
    /// Bamboo cap with a dangling ribbon (`hat01`).
    Bamboo,
    /// Wide straw brim (`hat02`), worn by boatmen.
    Brim,
}

/// What a person holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    None,
    /// The boatman's punting pole (`stick01`).
    Pole,
}

/// Maps a shape given in bone units (x across, y along) onto the bone
/// `p0 → p1`, mirrored like the original (`tranpoly`).
fn on_bone(p0: Pt, p1: Pt, shape: &[Pt], fli: bool) -> Vec<Pt> {
    let ang = (p1[1] - p0[1]).atan2(p1[0] - p0[0]) - PI / 2.0;
    let scl = distance(p0, p1);
    let sx = if fli { 1.0 } else { -1.0 };
    shape
        .iter()
        .map(|v| {
            let v = [sx * v[0], v[1]];
            let (d, a) = (v[0].hypot(v[1]), v[1].atan2(v[0]));
            [
                p0[0] + d * scl * (ang + a).cos(),
                p0[1] + d * scl * (ang + a).sin(),
            ]
        })
        .collect()
}

impl Hat {
    pub fn draw(self, sketch: &mut Sketch, chance: &mut Chance, neck: Pt, head: Pt, fli: bool) {
        let ink = Paint::ink(100, 0.8);
        match self {
            Self::Bamboo => {
                let cap = [
                    [-0.3, 0.5],
                    [0.3, 0.8],
                    [0.2, 1.0],
                    [0.0, 1.1],
                    [-0.3, 1.15],
                    [-0.55, 1.0],
                    [-0.65, 0.5],
                ];
                sketch.poly(on_bone(neck, head, &cap, fli), PolyStyle::filled(ink));
                let seed = chance.random();
                let ribbon: Vec<Pt> = (0..10)
                    .map(|i| {
                        [
                            -0.3 - chance.noise2(i as f64 * 0.2, seed) * i as f64 * 0.1,
                            0.5 - i as f64 * 0.3,
                        ]
                    })
                    .collect();
                sketch.poly(
                    on_bone(neck, head, &ribbon, fli),
                    PolyStyle::outlined(ink, 1.0),
                );
            }
            Self::Brim => {
                let brim = [
                    [-0.3, 0.5],
                    [-1.1, 0.5],
                    [-1.2, 0.6],
                    [-1.1, 0.7],
                    [-0.3, 0.8],
                    [0.3, 0.8],
                    [1.0, 0.7],
                    [1.3, 0.6],
                    [1.2, 0.5],
                    [0.3, 0.5],
                ];
                sketch.poly(on_bone(neck, head, &brim, fli), PolyStyle::filled(ink));
            }
        }
    }
}

impl Item {
    pub fn draw(self, sketch: &mut Sketch, chance: &mut Chance, hand: Pt, other: Pt, fli: bool) {
        if self == Self::None {
            return;
        }
        let seed = chance.random();
        let l = 12.0;
        let pole: Vec<Pt> = (0..12)
            .map(|i| {
                let t = i as f64;
                [
                    -chance.noise2(t * 0.1, seed) * 0.1 * (t / l * PI).sin() * 5.0,
                    t * 0.3,
                ]
            })
            .collect();
        sketch.poly(
            on_bone(hand, other, &pole, fli),
            PolyStyle::outlined(Paint::ink(100, 0.5), 1.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_bone_maps_axis() {
        let pts = on_bone([0.0, 0.0], [0.0, -10.0], &[[0.0, 1.0]], false);
        assert!(
            (pts[0][0]).abs() < 1e-9 && (pts[0][1] + 10.0).abs() < 1e-9,
            "{pts:?}"
        );
    }

    #[test]
    fn hats_and_items_draw() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        Hat::Bamboo.draw(&mut s, &mut c, [0.0, 0.0], [0.0, -10.0], false);
        Hat::Brim.draw(&mut s, &mut c, [0.0, 0.0], [0.0, -10.0], true);
        Item::Pole.draw(&mut s, &mut c, [0.0, 0.0], [10.0, 0.0], false);
        Item::None.draw(&mut s, &mut c, [0.0, 0.0], [10.0, 0.0], false);
        assert_eq!(s.len(), 4);
    }
}

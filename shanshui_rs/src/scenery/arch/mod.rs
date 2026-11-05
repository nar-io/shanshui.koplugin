//! Architecture (`Arch.*`): pavilions, houses, pagodas, boats and pylons.

mod boat;
mod buildings;
mod parts;
mod roofs;
mod tower;

pub use boat::boat01;
pub use buildings::{Arch01Args, Arch02Args, arch01, arch02, arch03, arch04};
pub use tower::transmission_tower;

use crate::brush::{StrokeStyle, flat_profile, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::ink::{Paint, Sketch};

/// A constant-width, noise-wobbled timber line (the strokes of `box`,
/// `roof` and `pagroof`).
fn beam(sketch: &mut Sketch, chance: &mut Chance, pts: &[Pt], origin: Pt, wei: f64, alpha: f64) {
    let style = StrokeStyle {
        col: Paint::ink(100, alpha),
        noi: 1.0,
        wid: wei,
        profile: &flat_profile,
        ..Default::default()
    };
    stroke(
        sketch,
        chance,
        &translated(pts, origin[0], origin[1]),
        &style,
    );
}

fn beams(
    sketch: &mut Sketch,
    chance: &mut Chance,
    lines: &[Vec<Pt>],
    origin: Pt,
    wei: f64,
    alpha: f64,
) {
    for line in lines {
        beam(sketch, chance, line, origin, wei, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beams_one_stroke_per_line() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        let lines = vec![vec![[0.0, 0.0], [5.0, 0.0], [10.0, 0.0]]; 3];
        beams(&mut s, &mut c, &lines, [1.0, 1.0], 2.0, 0.4);
        assert_eq!(s.len(), 3);
    }
}

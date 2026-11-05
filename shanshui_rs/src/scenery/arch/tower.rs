//! `Arch.transmissionTower01`: a lattice pylon.

use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::geom::subdivide::subdivide;
use crate::ink::{Paint, Sketch};

const HEI: f64 = 100.0;
const WID: f64 = 20.0;

/// Example: `transmission_tower(&mut sk, &mut ch, x, y)`.
pub fn transmission_tower(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64) {
    let mut lines = cross_arms();
    let left = [
        [-WID * 0.05, -HEI],
        [-WID * 0.1, -HEI * 0.9],
        [-WID * 0.2, -HEI * 0.5],
        [-WID * 0.5, 0.0],
    ];
    let right: Vec<Pt> = left.iter().map(|p| [-p[0], p[1]]).collect();
    let (l10, l11) = (subdivide(&left, 5), subdivide(&right, 5));
    for i in 0..l10.len() - 1 {
        lines.push(vec![l10[i], l11[i + 1]]);
        lines.push(vec![l11[i], l10[i + 1]]);
    }
    for k in 0..3 {
        lines.push(vec![left[k], right[k]]);
    }
    lines.push(left.to_vec());
    lines.push(right);
    let half = |_: f64| 0.5;
    let style = StrokeStyle {
        wid: 1.0,
        profile: &half,
        col: Paint::ink(100, 0.4),
        ..Default::default()
    };
    for line in lines {
        stroke(
            sketch,
            chance,
            &translated(&subdivide(&line, 5), xoff, yoff),
            &style,
        );
    }
}

/// Three cross-arms with their insulator strings.
fn cross_arms() -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    for [bx, by] in [[0.7, -0.85], [1.0, -0.675], [0.7, -0.5]] {
        let (l, r) = ([-bx * WID, by * HEI], [bx * WID, by * HEI]);
        let peak = [0.0, (by - 0.05) * HEI];
        out.push(vec![l, r]);
        out.push(vec![l, peak]);
        out.push(vec![r, peak]);
        out.push(vec![l, [l[0], (by + 0.1) * HEI]]);
        out.push(vec![r, [r[0], (by + 0.1) * HEI]]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tower_stroke_count() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        transmission_tower(&mut s, &mut c, 0.0, 0.0);
        assert_eq!(s.len(), 15 + 30 + 3 + 2);
    }

    #[test]
    fn tower_stands_on_origin() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        transmission_tower(&mut s, &mut c, 50.0, 200.0);
        let b = s.bbox().unwrap();
        assert!(b[3] <= 201.0 && b[1] >= 99.0, "{b:?}");
    }
}

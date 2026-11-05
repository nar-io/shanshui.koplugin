//! `foot`: the skirts at the base of a mountain — short paper-filled flaps
//! that hide the lower contours and give the slopes their layered feet.

use super::Mesh;
use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::ink::{Paint, PolyStyle, Sketch};

const SPAN: usize = 10;

/// Example: `foot(&mut sk, &mut ch, &mesh, [xoff, yoff])`.
pub fn foot(sketch: &mut Sketch, chance: &mut Chance, mesh: &Mesh, origin: Pt) {
    let flaps = flaps(chance, mesh, origin[0]);
    for f in &flaps {
        sketch.poly(translated(f, origin[0], origin[1]), PolyStyle::paper());
    }
    for f in &flaps {
        let style = StrokeStyle {
            col: Paint::ink(100, 0.1 + chance.random() * 0.1),
            wid: 1.0,
            ..Default::default()
        };
        stroke(sketch, chance, &translated(f, origin[0], origin[1]), &style);
    }
}

/// A left and a right flap every one or two contours.
fn flaps(chance: &mut Chance, mesh: &Mesh, xoff: f64) -> Vec<Vec<Pt>> {
    let mut out = Vec::new();
    let mut next = 0;
    for i in 0..mesh.len().saturating_sub(2) {
        if i != next {
            continue;
        }
        next = (next + chance.choice(&[1, 2])).min(mesh.len() - 1);
        let (mut left, mut right) = edge_runs(chance, &mesh[i], i);
        let wobble = chance.noise2(xoff * 0.05, i as f64) * 5.0;
        for j in 0..SPAN {
            let p = j as f64 / SPAN as f64;
            let vib = -1.7 * (p - 1.0) * p.powf(0.2) * 5.0 + wobble;
            left.push(drop_to(mesh[i][0], mesh[next][0], p, vib));
            let last = mesh[i].len() - 1;
            right.push(drop_to(mesh[i][last], mesh[next][last], p, vib));
        }
        out.push(left);
        out.push(right);
    }
    out
}

/// The first few points at each end of a contour, nudged inwards, reversed.
fn edge_runs(chance: &Chance, row: &[Pt], i: usize) -> (Vec<Pt>, Vec<Pt>) {
    let n = row.len().div_ceil(8).min(10);
    let shift = |j: usize| chance.noise2(j as f64 * 0.1, i as f64) * 10.0;
    let left = (0..n)
        .rev()
        .map(|j| [row[j][0] + shift(j), row[j][1]])
        .collect();
    let right = (0..n)
        .rev()
        .map(|j| {
            [
                row[row.len() - 1 - j][0] - shift(j),
                row[row.len() - 1 - j][1],
            ]
        })
        .collect();
    (left, right)
}

fn drop_to(a: Pt, b: Pt, p: f64, dy: f64) -> Pt {
    [
        a[0] * (1.0 - p) + b[0] * p,
        a[1] * (1.0 - p) + b[1] * p + dy,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh() -> Mesh {
        (0..10)
            .map(|i| {
                (0..50)
                    .map(|j| [(j as f64 - 25.0) * (10.0 - i as f64), -(i as f64) * 5.0])
                    .collect()
            })
            .collect()
    }

    #[test]
    fn flaps_come_in_pairs() {
        let mut c = Chance::from_seed(1);
        let f = flaps(&mut c, &mesh(), 0.0);
        assert!(!f.is_empty() && f.len().is_multiple_of(2));
        assert!(f.iter().all(|flap| flap.len() == 7 + SPAN));
    }

    #[test]
    fn foot_paper_before_strokes() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        foot(&mut s, &mut c, &mesh(), [0.0, 0.0]);
        let half = s.len() / 2;
        assert!(
            s.polygons[..half]
                .iter()
                .all(|p| p.style.fill == Paint::Paper)
        );
    }
}

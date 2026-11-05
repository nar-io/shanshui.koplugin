//! `barkify`: knots, scars and long ridged lines on a limb given by its two
//! edges.

use std::f64::consts::PI;

use crate::brush::{BlobStyle, StrokeStyle, blob, blob_outline, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::{lerp, translated};
use crate::geom::subdivide::subdivide;
use crate::ink::{Paint, Sketch};

/// Example: `barkify(&mut sk, &mut ch, [x, y], &branch(&mut ch, &args))`.
pub fn barkify(sketch: &mut Sketch, chance: &mut Chance, origin: Pt, limb: &[Vec<Pt>; 2]) {
    let [l, r] = limb;
    for i in 2..l.len().min(r.len()).saturating_sub(1) {
        let a0 = (l[i][1] - l[i - 1][1]).atan2(l[i][0] - l[i - 1][0]);
        let a1 = (r[i][1] - r[i - 1][1]).atan2(r[i][0] - r[i - 1][0]);
        let p = chance.random();
        let at = lerp(l[i], r[i], p);
        let at = [at[0] + origin[0], at[1] + origin[1]];
        mark(sketch, chance, at, p, (a0 + a1) / 2.0);
        if chance.random() < 0.05 {
            let (base, a) = if chance.random() < 0.5 {
                (l[i], a0)
            } else {
                (r[i], a1)
            };
            scar(
                sketch,
                chance,
                [base[0] + origin[0], base[1] + origin[1]],
                a,
                a0,
            );
        }
    }
    ridges(sketch, chance, origin, limb);
}

/// A knot (20%) or a short bark crack (80%) at `at`.
fn mark(sketch: &mut Sketch, chance: &mut Chance, at: Pt, p: f64, ang: f64) {
    let off_centre = (p - 0.5).abs() * 10.0;
    if chance.random() < 0.2 {
        let style = BlobStyle {
            noi: 1.0,
            len: 15.0,
            wid: 6.0 - off_centre,
            ang,
            col: Paint::ink(100, 0.6),
            ..Default::default()
        };
        blob(sketch, chance, at[0], at[1], &style);
    } else {
        crack(sketch, chance, at, 5.0 - off_centre, ang);
    }
}

/// The original's inner `bark()`: a blob outline traced with a dashed,
/// wavy thin stroke.
fn crack(sketch: &mut Sketch, chance: &mut Chance, at: Pt, wid: f64, ang: f64) {
    let len = 10.0 + 10.0 * chance.random();
    let outline = blob_outline(
        chance,
        at[0],
        at[1],
        &BlobStyle {
            len,
            wid,
            ang,
            ..Default::default()
        },
    );
    let fr = chance.random();
    let wave = move |x: f64| ((x + fr) * PI * 3.0).sin();
    let style = StrokeStyle {
        wid: 0.8,
        noi: 0.0,
        col: Paint::ink(100, 0.4),
        out: 0.0,
        profile: &wave,
    };
    stroke(sketch, chance, &outline, &style);
}

/// A row of small dabs across the limb edge.
fn scar(sketch: &mut Sketch, chance: &mut Chance, base: Pt, a: f64, a0: f64) {
    let jl = chance.random() * 2.0 + 2.0;
    let mut j = 0.0;
    while j < jl {
        let d = (j - jl / 2.0) * 4.0;
        let len = 4.0 + 6.0 * chance.random();
        let style = BlobStyle {
            wid: 4.0,
            len,
            ang: a0 + PI / 2.0,
            col: Paint::ink(100, 0.6),
            ..Default::default()
        };
        blob(
            sketch,
            chance,
            base[0] + a.cos() * d,
            base[1] + a.sin() * d,
            &style,
        );
        j += 1.0;
    }
}

/// Long ridge lines: the outline cut into random runs, each subdivided
/// and pushed around by noise.
fn ridges(sketch: &mut Sketch, chance: &mut Chance, origin: Pt, limb: &[Vec<Pt>; 2]) {
    let outline: Vec<Pt> = limb[0]
        .iter()
        .chain(limb[1].iter().rev())
        .copied()
        .collect();
    let mut runs: Vec<Vec<Pt>> = vec![Vec::new()];
    for p in outline {
        if chance.random() < 0.5 {
            runs.push(Vec::new());
        } else if let Some(run) = runs.last_mut() {
            run.push(p);
        }
    }
    let style = StrokeStyle {
        wid: 1.5,
        col: Paint::ink(100, 0.7),
        out: 0.0,
        ..Default::default()
    };
    for (i, run) in runs.iter().enumerate() {
        let mut line = subdivide(run, 4);
        for (j, q) in line.iter_mut().enumerate() {
            let (fi, fj) = (i as f64, j as f64 * 0.1);
            q[0] += (chance.noise(fi, fj, 1.0) - 0.5) * (15.0 + 5.0 * chance.gaussian());
            q[1] += (chance.noise(fi, fj, 2.0) - 0.5) * (15.0 + 5.0 * chance.gaussian());
        }
        stroke(
            sketch,
            chance,
            &translated(&line, origin[0], origin[1]),
            &style,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limb() -> [Vec<Pt>; 2] {
        let l = (0..20).map(|i| [-3.0, -(i as f64) * 5.0]).collect();
        let r = (0..20).map(|i| [3.0, -(i as f64) * 5.0]).collect();
        [l, r]
    }

    #[test]
    fn barkify_marks_the_limb() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        barkify(&mut s, &mut c, [100.0, 100.0], &limb());
        assert!(s.len() >= 17, "only {} marks", s.len());
        let b = s.bbox().unwrap();
        assert!(b[0] > 60.0 && b[2] < 140.0, "bbox {b:?}");
    }

    #[test]
    fn short_limb_only_ridges() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        barkify(
            &mut s,
            &mut c,
            [0.0, 0.0],
            &[vec![[0.0, 0.0]], vec![[1.0, 0.0]]],
        );
        assert!(s.len() <= 2);
    }
}

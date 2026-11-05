//! tree08: a leaning sapling covered in fine fractal sprigs.

use std::f64::consts::PI;

use super::branch::{BranchArgs, branch};
use super::ink_trunk;
use crate::brush::{StrokeStyle, flat_profile, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::{distance, joined_reversed};
use crate::geom::subdivide::subdivide;
use crate::ink::{Paint, Sketch};

/// Example: `tree08(&mut sk, &mut ch, x, y, 80.0)`.
pub fn tree08(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64, hei: f64) {
    let lean = chance.norm_rand(-1.0, 1.0) * PI * 0.2;
    let args = BranchArgs {
        hei,
        wid: 1.0,
        ang: -PI / 2.0 + lean,
        ben: PI * 0.2,
        det: hei / 20.0,
    };
    let edges = branch(chance, &args);
    let trunk = joined_reversed(&edges[0], &edges[1]);
    let mut sprigs = Sketch::new();
    for (i, p) in trunk.iter().enumerate() {
        let at = [x + p[0], y + p[1]];
        if chance.random() < 0.2 {
            let dep = (4.0 * chance.random()).floor() as u32;
            let ang = -PI / 2.0 - lean * chance.random();
            sprig(
                &mut sprigs,
                chance,
                at,
                dep,
                Sprig {
                    ang,
                    len: 15.0,
                    ben: 0.0,
                },
            );
        } else if i == trunk.len() / 2 {
            sprig(
                &mut sprigs,
                chance,
                at,
                3,
                Sprig {
                    ang: -PI / 2.0 + lean,
                    len: 15.0,
                    ben: 0.0,
                },
            );
        }
    }
    ink_trunk(sketch, chance, [x, y], &trunk, 0.6, false);
    sketch.append(sprigs);
}

#[derive(Clone, Copy)]
struct Sprig {
    ang: f64,
    len: f64,
    ben: f64,
}

/// One slightly arched segment from `start`, then one or two children.
fn sprig(sketch: &mut Sketch, chance: &mut Chance, start: Pt, dep: u32, s: Sprig) {
    let line = arched_segment(chance, start, s);
    let tapered = |x: f64| (0.5 * PI * x).cos();
    let profile: &dyn Fn(f64) -> f64 = if dep == 0 { &tapered } else { &flat_profile };
    let style = StrokeStyle {
        profile,
        wid: 0.8,
        col: Paint::ink(100, 0.5),
        ..Default::default()
    };
    stroke(sketch, chance, &line, &style);
    if dep == 0 {
        return;
    }
    let end = [
        start[0] + s.ang.cos() * s.len,
        start[1] + s.ang.sin() * s.len,
    ];
    let df = f64::from(dep);
    let ben = s.ben + chance.sign() * PI * 0.001 * df * df;
    for spread in children(chance) {
        let len = s.len * chance.norm_rand(0.8, 0.9);
        sprig(
            sketch,
            chance,
            end,
            dep - 1,
            Sprig {
                ang: s.ang + s.ben + PI * spread * 0.2,
                len,
                ben,
            },
        );
    }
}

/// Angular offsets of the children: a fork (half the time) or one shoot.
fn children(chance: &mut Chance) -> Vec<f64> {
    if chance.random() >= 0.5 {
        return vec![0.0];
    }
    let mut pick = |a: (f64, f64), b: (f64, f64)| {
        let (x, y) = (chance.norm_rand(a.0, a.1), chance.norm_rand(b.0, b.1));
        if chance.random() < 0.5 { x } else { y }
    };
    vec![
        pick((-1.0, 0.5), (0.5, 1.0)),
        pick((-1.0, -0.5), (0.5, 1.0)),
    ]
}

fn arched_segment(chance: &mut Chance, start: Pt, s: Sprig) -> Vec<Pt> {
    let flip = chance.sign();
    let mut line = subdivide(&[start, [start[0] + s.len, start[1]]], 10);
    let n = line.len() as f64;
    for (i, p) in line.iter_mut().enumerate() {
        p[1] += flip * (i as f64 / n * PI).sin() * 2.0;
        let (d, a) = (
            distance(*p, start),
            (p[1] - start[1]).atan2(p[0] - start[0]),
        );
        *p = [
            start[0] + d * (a + s.ang).cos(),
            start[1] + d * (a + s.ang).sin(),
        ];
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sapling_draws() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        tree08(&mut s, &mut c, 0.0, 0.0, 80.0);
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert!(s.len() > 5);
    }

    #[test]
    fn arched_segment_points_along_angle() {
        let mut c = Chance::from_seed(2);
        let line = arched_segment(
            &mut c,
            [0.0, 0.0],
            Sprig {
                ang: -PI / 2.0,
                len: 10.0,
                ben: 0.0,
            },
        );
        let end = line[line.len() - 1];
        // The arch (±2) does not quite return to zero at the last sample.
        assert!(
            end[0].abs() < 1.0 && (end[1] + 10.0).abs() < 0.1,
            "end {end:?}"
        );
    }

    #[test]
    fn leaf_sprig_is_one_stroke() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        sprig(
            &mut s,
            &mut c,
            [0.0, 0.0],
            0,
            Sprig {
                ang: 0.0,
                len: 15.0,
                ben: 0.0,
            },
        );
        assert_eq!(s.len(), 1);
    }
}

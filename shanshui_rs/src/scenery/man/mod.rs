//! Little people (`Man.*`): a 9-joint skeleton dressed in robe, sleeves,
//! head and hat, optionally holding a pole.
//!
//! ```text
//!      2          joints: 0 hip, 1 neck, 2 head, 3 knee, 4 foot,
//!    1/                   5 elbow, 6 hand, 7 elbow, 8 hand
//! 7/  | \_ 6
//! 8| 0 \ 5
//!      /3
//!     4
//! ```

mod expand;
mod hats;

use std::f64::consts::PI;

pub use hats::{Hat, Item};

use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::bezier::bezmh;
use crate::ink::{Paint, PolyStyle, Sketch};
use expand::expand;

/// Parent joint of each joint (the original's nested `sct` object).
const PARENT: [Option<usize>; 9] = [
    None,
    Some(0),
    Some(1),
    Some(0),
    Some(3),
    Some(1),
    Some(5),
    Some(1),
    Some(7),
];

/// Arguments of `man(x, y, args)`.
#[derive(Clone, Copy, Debug)]
pub struct ManArgs {
    pub sca: f64,
    pub hat: Hat,
    pub item: Item,
    /// Mirror horizontally (faces left when `true`).
    pub fli: bool,
    /// Joint angles relative to the parent joint.
    pub ang: [f64; 9],
    /// Bone lengths before scaling.
    pub len: [f64; 9],
}

impl ManArgs {
    /// Defaults, with the random pose the original draws for each person.
    pub fn new(chance: &mut Chance) -> Self {
        let ang = [
            0.0,
            -PI / 2.0,
            0.0,
            PI / 4.0 * chance.random(),
            PI * 3.0 / 4.0 * chance.random(),
            PI * 3.0 / 4.0,
            -PI / 4.0,
            -PI * 3.0 / 4.0 - PI / 4.0 * chance.random(),
            -PI / 4.0,
        ];
        let len = [0.0, 30.0, 20.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0];
        Self {
            sca: 0.5,
            hat: Hat::Bamboo,
            item: Item::None,
            fli: true,
            ang,
            len,
        }
    }
}

/// Joints from the root down to `ind`.
fn chain(ind: usize) -> Vec<usize> {
    let mut out = vec![ind];
    while let Some(p) = PARENT[out[out.len() - 1]] {
        out.push(p);
    }
    out.reverse();
    out
}

/// Joint positions in body space (hip at the origin).
fn pose(a: &ManArgs) -> [Pt; 9] {
    let rot = |k: usize| chain(k).iter().map(|&j| a.ang[j]).sum::<f64>();
    std::array::from_fn(|ind| {
        chain(ind).iter().fold([0.0, 0.0], |p, &k| {
            let l = a.len[k] * a.sca;
            [p[0] + l * rot(k).cos(), p[1] + l * rot(k).sin()]
        })
    })
}

/// Example: `man(&mut sk, &mut ch, x, y, &ManArgs { sca: 0.42, ..ManArgs::new(&mut ch) })`.
pub fn man(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, a: &ManArgs) {
    let pts = pose(a);
    let flip = if a.fli { -1.0 } else { 1.0 };
    let yoff = yoff - pts[4][1];
    let g = |v: Pt| [flip * v[0] + xoff, v[1] + yoff];
    a.item.draw(sketch, chance, g(pts[8]), g(pts[6]), a.fli);
    let sca = a.sca;
    let sleeve = move |x: f64| {
        sca * 8.0 * ((0.5 * x * PI).sin() * (x * PI).sin().max(0.0).powf(0.1) + (1.0 - x) * 0.4)
    };
    let body = move |x: f64| {
        sca * 11.0 * ((0.5 * x * PI).sin() * (x * PI).sin().max(0.0).powf(0.1) + (1.0 - x) * 0.5)
    };
    let head = move |x: f64| sca * 7.0 * (0.25 - (x - 0.5).powi(2)).max(0.0).powf(0.3);
    cloth(sketch, chance, &[pts[1], pts[7], pts[8]], &sleeve, &g);
    cloth(sketch, chance, &[pts[1], pts[0], pts[3], pts[4]], &body, &g);
    cloth(sketch, chance, &[pts[1], pts[5], pts[6]], &sleeve, &g);
    cloth(sketch, chance, &[pts[1], pts[2]], &head, &g);
    hair(sketch, &[pts[1], pts[2]], &head, &g);
    a.hat.draw(sketch, chance, g(pts[1]), g(pts[2]), a.fli);
}

/// A garment piece around a limb: paper fill plus two edge strokes.
fn cloth(
    sketch: &mut Sketch,
    chance: &mut Chance,
    limb: &[Pt],
    width: &dyn Fn(f64) -> f64,
    g: &dyn Fn(Pt) -> Pt,
) {
    let (l, r) = expand(&bezmh(limb, 2.0), width);
    let fill: Vec<Pt> = l.iter().chain(r.iter().rev()).map(|p| g(*p)).collect();
    sketch.poly(fill, PolyStyle::paper());
    for (edge, alpha) in [(&l, 0.5), (&r, 0.6)] {
        let line: Vec<Pt> = edge.iter().map(|p| g(*p)).collect();
        stroke(
            sketch,
            chance,
            &line,
            &StrokeStyle {
                wid: 1.0,
                col: Paint::ink(100, alpha),
                ..Default::default()
            },
        );
    }
}

/// The dark back of the head.
fn hair(sketch: &mut Sketch, limb: &[Pt], width: &dyn Fn(f64) -> f64, g: &dyn Fn(Pt) -> Pt) {
    let (l, r) = expand(&bezmh(limb, 2.0), width);
    let l = &l[l.len() / 10..];
    let r = &r[(r.len() as f64 * 0.95) as usize..];
    let fill: Vec<Pt> = l.iter().chain(r.iter().rev()).map(|p| g(*p)).collect();
    sketch.poly(fill, PolyStyle::filled(Paint::ink(100, 0.6)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains_reach_root() {
        assert_eq!(chain(0), vec![0]);
        assert_eq!(chain(6), vec![0, 1, 5, 6]);
        assert_eq!(chain(4), vec![0, 3, 4]);
    }

    #[test]
    fn neck_above_hip() {
        let mut c = Chance::from_seed(1);
        let pts = pose(&ManArgs::new(&mut c));
        assert!(pts[1][1] < pts[0][1] - 10.0, "{:?}", pts);
        assert!(pts[2][1] < pts[1][1]);
    }

    #[test]
    fn man_stands_on_given_y() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        let a = ManArgs::new(&mut c);
        man(&mut s, &mut c, 100.0, 300.0, &a);
        let b = s.bbox().unwrap();
        assert!(b[3] < 310.0 && b[3] > 290.0, "{b:?}");
        assert!(s.len() > 8);
    }
}

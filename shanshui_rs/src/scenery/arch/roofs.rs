//! `roof` (gabled, curled eaves) and `pagroof` (pagoda cap), plus `rail`
//! (balustrades).

use super::beams;
use crate::brush::{StrokeStyle, flat_profile, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::geom::subdivide::subdivide;
use crate::ink::{Paint, PolyStyle, Sketch};

/// Shared shape arguments of roofs and rails.
#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub hei: f64,
    pub wid: f64,
    pub rot: f64,
    pub per: f64,
    pub wei: f64,
}

/// Gabled roof whose ridge sits `hei` above `(xoff, yoff)`.
/// (The original's "Pizza Hut" signboard is text and is left out.)
pub fn roof(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, s: &Span) {
    let flip = s.rot < 0.5;
    let opf = |pts: Vec<Pt>| {
        if flip {
            pts.iter().map(|p| [-p[0], p[1]]).collect()
        } else {
            pts
        }
    };
    let rrot = if flip { 1.0 - s.rot } else { s.rot };
    let (hw, hei, per, cor) = (s.wid * 0.5, s.hei, s.per, 5.0);
    let mid = -hw + s.wid * rrot;
    let quat = (mid + hw) * 0.5 - mid;
    let lines = [
        vec![
            [-hw + quat, -hei - per / 2.0],
            [-hw + quat * 0.5, -hei / 2.0 - per / 4.0],
            [-hw - cor, 0.0],
        ],
        vec![
            [mid + quat, -hei],
            [(mid + quat + hw) / 2.0, -hei / 2.0],
            [hw + cor, 0.0],
        ],
        vec![
            [mid + quat, -hei],
            [mid + quat / 2.0, -hei / 2.0 + per / 2.0],
            [mid + cor, per],
        ],
        vec![[-hw - cor, 0.0], [mid + cor, per]],
        vec![[hw + cor, 0.0], [mid + cor, per]],
        vec![[-hw + quat, -hei - per / 2.0], [mid + quat, -hei]],
    ];
    let outline = opf(vec![
        [-hw, 0.0],
        [-hw + quat, -hei - per / 2.0],
        [mid + quat, -hei],
        [hw, 0.0],
        [mid, per],
    ]);
    sketch.poly(translated(&outline, xoff, yoff), PolyStyle::paper());
    let lines: Vec<Vec<Pt>> = lines.into_iter().map(|l| subdivide(&opf(l), 5)).collect();
    beams(sketch, chance, &lines, [xoff, yoff], s.wei, 0.4);
}

/// Four-sided pagoda cap with flared corners.
pub fn pagroof(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, s: &Span) {
    let (sid, cor) = (4, 10.0);
    let mut lines: Vec<Vec<Pt>> = Vec::new();
    let mut outline = vec![[0.0, -s.hei]];
    for i in 0..sid {
        let t = i as f64 / (sid - 1) as f64 - 0.5;
        let (fx, fy, fxx) = (s.wid * t, s.per * (1.0 - t.abs() * 2.0), (s.wid + cor) * t);
        if let Some(prev) = lines.last() {
            let corner = prev[2];
            lines.push(vec![corner, [fxx, fy]]);
        }
        lines.push(vec![
            [0.0, -s.hei],
            [fx * 0.5, (-s.hei + fy) * 0.5],
            [fxx, fy],
        ]);
        outline.push([fxx, fy]);
    }
    sketch.poly(translated(&outline, xoff, yoff), PolyStyle::paper());
    let lines: Vec<Vec<Pt>> = lines.iter().map(|l| subdivide(l, 5)).collect();
    beams(sketch, chance, &lines, [xoff, yoff], s.wei, 0.4);
}

/// Arguments of `rail(x, y, seed, args)` beyond the shared span.
#[derive(Clone, Copy, Debug)]
pub struct RailArgs {
    pub span: Span,
    pub seg: usize,
    /// Back rail (seen through the building).
    pub tra: bool,
    /// Front rail.
    pub fro: bool,
}

/// A balustrade: top and bottom bars joined by wobbly balusters.
pub fn rail(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    seed: f64,
    a: &RailArgs,
) {
    let mut bars = rail_bars(a);
    if a.tra && !bars.is_empty() {
        // The original trims the same bar twice (`(open + n) % n == open`).
        let open = (chance.random() * bars.len() as f64) as usize % bars.len();
        let keep = bars[open].len().saturating_sub(2);
        bars[open].truncate(keep);
    }
    balusters(sketch, chance, &mut bars, [xoff, yoff], seed, a.span.hei);
    let style = StrokeStyle {
        col: Paint::ink(100, 0.5),
        noi: 0.5,
        wid: a.span.wei,
        profile: &flat_profile,
        ..Default::default()
    };
    for bar in &bars {
        stroke(sketch, chance, &translated(bar, xoff, yoff), &style);
    }
}

/// Bottom bars first, then top bars, so bar `i` pairs with `i + n/2`.
fn rail_bars(a: &RailArgs) -> Vec<Vec<Pt>> {
    let s = &a.span;
    let (hw, mid, bmid) = (
        s.wid * 0.5,
        -s.wid * 0.5 + s.wid * s.rot,
        -s.wid * 0.5 + s.wid * (1.0 - s.rot),
    );
    let mut bars = Vec::new();
    for dy in [0.0, -s.hei] {
        if a.fro {
            bars.push(subdivide(&[[-hw, dy], [mid, dy + s.per]], a.seg));
            bars.push(subdivide(&[[mid, dy + s.per], [hw, dy]], a.seg));
        }
        if a.tra {
            bars.push(subdivide(&[[-hw, dy], [bmid, dy - s.per]], a.seg));
            bars.push(subdivide(&[[bmid, dy - s.per], [hw, dy]], a.seg));
        }
    }
    bars
}

fn balusters(
    sketch: &mut Sketch,
    chance: &mut Chance,
    bars: &mut [Vec<Pt>],
    origin: Pt,
    seed: f64,
    hei: f64,
) {
    let n = bars.len();
    for i in 0..n / 2 {
        let k = (n / 2 + i) % n;
        for j in 0..bars[i].len() {
            bars[i][j][1] += (chance.noise(i as f64, j as f64 * 0.5, seed) - 0.5) * hei;
            let jj = j % bars[k].len().max(1);
            bars[k][jj][1] += (chance.noise(i as f64 + 0.5, j as f64 * 0.5, seed) - 0.5) * hei;
            let mut post = subdivide(&[bars[i][j], bars[k][jj]], 2);
            post[0][0] += (chance.random() - 0.5) * hei * 0.5;
            sketch.poly(
                translated(&post, origin[0], origin[1]),
                PolyStyle::outlined(Paint::ink(100, 0.5), 2.0),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPAN: Span = Span {
        hei: 10.0,
        wid: 50.0,
        rot: 0.7,
        per: 5.0,
        wei: 1.5,
    };

    #[test]
    fn roof_paper_first() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        roof(&mut s, &mut c, 0.0, 0.0, &SPAN);
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert_eq!(s.len(), 7);
    }

    #[test]
    fn flipped_roof_mirrors() {
        let (mut a, mut b, mut c) = (Sketch::new(), Sketch::new(), Chance::from_seed(2));
        roof(&mut a, &mut c, 0.0, 0.0, &SPAN);
        roof(&mut b, &mut c, 0.0, 0.0, &Span { rot: 0.3, ..SPAN });
        assert_eq!(a.polygons[0].pts[1][0], -b.polygons[0].pts[1][0]);
    }

    #[test]
    fn pagroof_has_seven_lines() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        pagroof(&mut s, &mut c, 0.0, 0.0, &SPAN);
        assert_eq!(s.len(), 1 + 7);
    }

    #[test]
    fn rail_bars_pair_up() {
        let a = RailArgs {
            span: SPAN,
            seg: 4,
            tra: true,
            fro: true,
        };
        assert_eq!(rail_bars(&a).len(), 8);
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(4));
        rail(&mut s, &mut c, 0.0, 0.0, 0.0, &a);
        assert!(s.len() > 8);
    }
}

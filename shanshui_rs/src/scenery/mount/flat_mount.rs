//! `Mount.flatMount`: a low plateau whose top is cut flat; the flat part
//! becomes a ground where `flat_dec` puts trees, rocks and huts.

use std::f64::consts::PI;

use super::flat_dec::{Ground, flat_dec};
use super::{Mesh, silhouette};
use crate::brush::{StrokeStyle, TextureStyle, stroke, texture};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::{bounds, translated};
use crate::geom::subdivide::subdivide;
use crate::ink::{Paint, PolyStyle, Sketch};

const RESO: [usize; 2] = [5, 50];

/// Arguments of `flatMount(x, y, seed, args)`.
#[derive(Clone, Copy, Debug)]
pub struct FlatMountArgs {
    pub hei: f64,
    pub wid: f64,
    pub tex: usize,
    /// Where the top is cut, as a fraction of 100 units.
    pub cho: f64,
}

/// Example: `flat_mount(&mut sk, &mut ch, x, y, 2.0, &FlatMountArgs { hei: 100.0, wid: 800.0, tex: 80, cho: 0.6 })`.
pub fn flat_mount(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    seed: f64,
    a: &FlatMountArgs,
) {
    let (mesh, flats) = plateau_mesh(chance, yoff, seed, a);
    silhouette(sketch, chance, &mesh[0], [xoff, yoff], RESO[0] as f64 * 4.0);
    let dis = |ch: &mut Chance| {
        if ch.random() > 0.5 {
            0.1 + 0.4 * ch.random()
        } else {
            0.9 - 0.4 * ch.random()
        }
    };
    let shifted: Mesh = mesh.iter().map(|row| translated(row, xoff, yoff)).collect();
    texture(
        sketch,
        chance,
        &shifted,
        &TextureStyle {
            tex: a.tex,
            wid: 2.0,
            dis: &dis,
            ..Default::default()
        },
    );
    let Some(ground) = ground_outline(chance, &flats) else {
        return;
    };
    sketch.poly(translated(&ground, xoff, yoff), PolyStyle::paper());
    let style = StrokeStyle {
        wid: 3.0,
        col: Paint::ink(100, 0.2),
        ..Default::default()
    };
    stroke(sketch, chance, &translated(&ground, xoff, yoff), &style);
    if let Some(b) = bounds(&ground) {
        flat_dec(
            sketch,
            chance,
            [xoff, yoff],
            &Ground {
                xmin: b[0],
                ymin: b[1],
                xmax: b[2],
                ymax: b[3],
            },
        );
    }
}

/// Contours plus, per contour, the start/end points of its clipped runs.
fn plateau_mesh(
    chance: &mut Chance,
    yoff: f64,
    seed: f64,
    a: &FlatMountArgs,
) -> (Mesh, Vec<Vec<Pt>>) {
    let (mut mesh, mut flats) = (Vec::new(), Vec::new());
    let mut hoff = 0.0;
    for j in 0..RESO[0] {
        hoff += chance.random() * yoff / 100.0;
        let p = 1.0 - j as f64 / RESO[0] as f64 * 0.6;
        let (row, flat) = plateau_row(chance, seed, a, j, p, hoff);
        mesh.push(row);
        flats.push(flat);
    }
    (mesh, flats)
}

fn plateau_row(
    chance: &Chance,
    seed: f64,
    a: &FlatMountArgs,
    j: usize,
    p: f64,
    hoff: f64,
) -> (Vec<Pt>, Vec<Pt>) {
    let cut = -100.0 * a.cho + hoff;
    let (mut row, mut flat): (Vec<Pt>, Vec<Pt>) = (Vec::new(), Vec::new());
    for i in 0..RESO[1] {
        let x = (i as f64 / RESO[1] as f64 - 0.5) * PI;
        let y = ((x * 2.0).cos() + 1.0) * chance.noise(x + 10.0, j as f64 * 0.1, seed);
        let nx = x / PI * a.wid * p;
        let mut ny = -y * a.hei * p + hoff;
        if ny < cut {
            ny = cut;
            if flat.len() % 2 == 0 {
                flat.push([nx, ny]);
            }
        } else if flat.len() % 2 == 1 {
            flat.push(*row.last().unwrap_or(&[nx, ny]));
        }
        row.push([nx, ny]);
    }
    (row, flat)
}

/// Closed outline of the flat top, tapered at both ends and roughened.
fn ground_outline(chance: &Chance, flats: &[Vec<Pt>]) -> Option<Vec<Pt>> {
    let (mut g1, mut g2): (Vec<Pt>, Vec<Pt>) = (Vec::new(), Vec::new());
    for f in flats.iter().step_by(2).filter(|f| f.len() >= 2) {
        g1.push(f[0]);
        g2.push(f[f.len() - 1]);
    }
    if g1.is_empty() {
        return None;
    }
    taper_ends(&mut g1);
    taper_ends(&mut g2);
    let (g1, g2) = (subdivide(&g1, 5), subdivide(&g2, 5));
    let mut ground: Vec<Pt> = g1.iter().rev().chain(g2.iter()).copied().collect();
    ground.push(g1[g1.len() - 1]);
    for (i, p) in ground.iter_mut().enumerate() {
        let v = (1.0 - ((i % 5) as f64 - 2.5).abs() / 2.5) * 0.12;
        p[0] *= 1.0 - v + chance.noise1(p[1] * 0.5) * v;
    }
    Some(ground)
}

/// Three points narrowing in above the first run and three below the last.
fn taper_ends(g: &mut Vec<Pt>) {
    let w = g[0][0];
    for i in 0..3 {
        let p = 0.8 - i as f64 * 0.2;
        g.insert(0, [w * p, g[0][1] - 5.0]);
    }
    let w = g[g.len() - 1][0];
    for i in 0..3 {
        let p = 0.6 - (i * i) as f64 * 0.1;
        g.push([w * p, g[g.len() - 1][1] + 1.0]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> FlatMountArgs {
        FlatMountArgs {
            hei: 100.0,
            wid: 800.0,
            tex: 80,
            cho: 0.5,
        }
    }

    #[test]
    fn top_is_clipped() {
        let mut c = Chance::from_seed(1);
        let (mesh, flats) = plateau_mesh(
            &mut c,
            600.0,
            1.0,
            &FlatMountArgs {
                hei: 400.0,
                ..args()
            },
        );
        let top = mesh[0].iter().map(|p| p[1]).fold(f64::MAX, f64::min);
        let at_top = mesh[0].iter().filter(|p| (p[1] - top).abs() < 1e-9).count();
        assert!(at_top > 1, "top not flat");
        assert!(flats[0].len() >= 2);
    }

    #[test]
    fn taper_adds_six_points() {
        let mut g = vec![[10.0, 0.0], [12.0, 5.0]];
        taper_ends(&mut g);
        assert_eq!(g.len(), 8);
        assert_eq!(g[2], [8.0, -5.0]);
    }

    #[test]
    fn flat_mount_draws() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        flat_mount(&mut s, &mut c, 0.0, 650.0, 2.0, &args());
        assert!(s.len() > 20, "only {}", s.len());
    }
}

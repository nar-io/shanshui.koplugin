//! `Mount.rock`: a squat boulder with heavy shading.

use std::f64::consts::PI;

use super::{Mesh, silhouette};
use crate::brush::{TextureStyle, texture};
use crate::chance::{Chance, loop_noise};
use crate::geom::point::translated;
use crate::ink::{Paint, Sketch};

const RESO: [usize; 2] = [10, 50];

/// Arguments of `rock(x, y, seed, args)`.
#[derive(Clone, Copy, Debug)]
pub struct RockArgs {
    pub hei: f64,
    pub wid: f64,
    pub tex: usize,
    pub sha: usize,
}

impl Default for RockArgs {
    fn default() -> Self {
        Self {
            hei: 80.0,
            wid: 100.0,
            tex: 40,
            sha: 10,
        }
    }
}

/// Example: `rock(&mut sk, &mut ch, x, y, 3.0, &RockArgs { wid: 30.0, hei: 25.0, sha: 2, ..Default::default() })`.
pub fn rock(
    sketch: &mut Sketch,
    chance: &mut Chance,
    xoff: f64,
    yoff: f64,
    seed: f64,
    a: &RockArgs,
) {
    let mesh = rock_mesh(chance, seed, a);
    silhouette(sketch, chance, &mesh[0], [xoff, yoff], 0.0);
    let col = |ch: &mut Chance, _: f64| Paint::ink(180, 0.3 + ch.random() * 0.3);
    let dis = |ch: &mut Chance| {
        if ch.random() > 0.5 {
            0.15 + 0.15 * ch.random()
        } else {
            0.85 - 0.15 * ch.random()
        }
    };
    let style = TextureStyle {
        tex: a.tex,
        wid: 3.0,
        sha: a.sha,
        col: &col,
        dis: &dis,
        ..Default::default()
    };
    let shifted: Mesh = mesh.iter().map(|row| translated(row, xoff, yoff)).collect();
    texture(sketch, chance, &shifted, &style);
}

/// Ellipse contours squashed below the horizon, roughened by looped noise.
fn rock_mesh(chance: &Chance, seed: f64, a: &RockArgs) -> Mesh {
    (0..RESO[0])
        .map(|i| {
            let mut ns: Vec<f64> = (0..RESO[1])
                .map(|j| chance.noise(i as f64, j as f64 * 0.2, seed))
                .collect();
            loop_noise(&mut ns);
            let p = 1.0 - i as f64 / RESO[0] as f64;
            (0..RESO[1])
                .map(|j| rock_point(a, j, ns[j], p, i))
                .collect()
        })
        .collect()
}

fn rock_point(a: &RockArgs, j: usize, ns: f64, p: f64, i: usize) -> [f64; 2] {
    let ang = j as f64 / RESO[1] as f64 * PI * 2.0 - PI / 2.0;
    let l = a.wid * a.hei / ((a.hei * ang.cos()).powi(2) + (a.wid * ang.sin()).powi(2)).sqrt();
    let l = l * (0.7 + 0.3 * ns);
    let mut ny = -ang.sin() * l * p;
    if !(0.0..=PI).contains(&ang) {
        ny *= 0.2;
    }
    [
        ang.cos() * l * p,
        ny + a.hei * (i as f64 / RESO[0] as f64) * 0.2,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rock_is_wider_than_tall() {
        let c = Chance::from_seed(1);
        let mesh = rock_mesh(
            &c,
            0.0,
            &RockArgs {
                wid: 40.0,
                hei: 30.0,
                ..Default::default()
            },
        );
        let xs: Vec<f64> = mesh[0].iter().map(|p| p[0]).collect();
        let span = xs.iter().cloned().fold(f64::MIN, f64::max)
            - xs.iter().cloned().fold(f64::MAX, f64::min);
        assert!(span > 40.0, "span {span}");
    }

    #[test]
    fn rock_draws_silhouette_and_texture() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        rock(&mut s, &mut c, 0.0, 0.0, 1.0, &RockArgs::default());
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
        assert!(s.len() > 5);
    }
}

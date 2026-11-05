//! `blob()`: a closed, noise-wobbled leaf/dab shape — leaves, moss, bark
//! knots.

use std::f64::consts::PI;

use crate::chance::{Chance, loop_noise};
use crate::geom::Pt;
use crate::ink::{Paint, PolyStyle, Sketch};

const RESO: usize = 20;

/// Arguments of `blob(x, y, args)`; `Default` mirrors the JS defaults.
#[derive(Clone, Copy)]
pub struct BlobStyle<'a> {
    pub len: f64,
    pub wid: f64,
    pub ang: f64,
    pub col: Paint,
    pub noi: f64,
    /// Half-width at `p ∈ [0, 2]`: upper edge for `p ≤ 1`, lower after.
    pub profile: &'a dyn Fn(f64) -> f64,
}

/// The default almond shape.
pub fn almond(p: f64) -> f64 {
    if p <= 1.0 {
        (p * PI).sin().powf(0.5)
    } else {
        -((p + 1.0) * PI).sin().powf(0.5)
    }
}

/// Leaf shape shared by tree02 and the twig leaves.
pub fn leaf(p: f64) -> f64 {
    if p <= 1.0 {
        ((p * PI).sin() * p).powf(0.5)
    } else {
        -((p - 2.0) * PI * (p - 2.0)).sin().powf(0.5)
    }
}

impl Default for BlobStyle<'_> {
    fn default() -> Self {
        Self {
            len: 20.0,
            wid: 5.0,
            ang: 0.0,
            col: Paint::ink(200, 0.9),
            noi: 0.5,
            profile: &almond,
        }
    }
}

/// Outline of a blob centred on `(x, y)` (the `ret: 1` mode).
pub fn blob_outline(chance: &mut Chance, x: f64, y: f64, style: &BlobStyle) -> Vec<Pt> {
    let n0 = chance.random() * 10.0;
    let mut ns: Vec<f64> = (0..=RESO)
        .map(|i| chance.noise2(i as f64 * 0.05, n0))
        .collect();
    loop_noise(&mut ns);
    (0..=RESO)
        .map(|i| {
            let (l, a) = polar(style, i);
            let k = ns[i] * style.noi + (1.0 - style.noi);
            [
                x + (a + style.ang).cos() * l * k,
                y + (a + style.ang).sin() * l * k,
            ]
        })
        .collect()
}

/// Length and angle of the i-th outline vertex before rotation and noise.
fn polar(style: &BlobStyle, i: usize) -> (f64, f64) {
    let p = i as f64 / RESO as f64 * 2.0;
    let xo = style.len / 2.0 - (p - 1.0).abs() * style.len;
    let yo = (style.profile)(p) * style.wid / 2.0;
    ((xo * xo + yo * yo).sqrt(), yo.atan2(xo))
}

/// Draws a filled blob. Example: `blob(&mut sk, &mut ch, x, y, &BlobStyle::default())`.
pub fn blob(sketch: &mut Sketch, chance: &mut Chance, x: f64, y: f64, style: &BlobStyle) {
    let pts = blob_outline(chance, x, y, style);
    sketch.poly(pts, PolyStyle::filled(style.col));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_has_reso_plus_one_points() {
        let mut c = Chance::from_seed(1);
        let pts = blob_outline(&mut c, 0.0, 0.0, &BlobStyle::default());
        assert_eq!(pts.len(), RESO + 1);
    }

    #[test]
    fn blob_stays_within_length() {
        let mut c = Chance::from_seed(2);
        let style = BlobStyle {
            len: 30.0,
            wid: 6.0,
            ..Default::default()
        };
        let pts = blob_outline(&mut c, 100.0, 50.0, &style);
        assert!(pts.iter().all(|p| (p[0] - 100.0).abs() <= 15.0 + 1e-9));
    }

    #[test]
    fn blob_draws_one_polygon() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        blob(&mut s, &mut c, 0.0, 0.0, &BlobStyle::default());
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn profiles_are_finite_on_domain() {
        for i in 0..=20 {
            let p = i as f64 / 10.0;
            assert!(almond(p).is_finite(), "almond({p})");
        }
    }
}

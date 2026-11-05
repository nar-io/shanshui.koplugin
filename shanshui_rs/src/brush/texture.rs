//! `texture()`: the cun (皴) shading strokes laid across a mountain or rock
//! mesh. `layers[i][j]` is the j-th point of the i-th contour, outermost
//! contour first.

use crate::brush::stroke::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::ink::{Paint, Sketch};

/// Arguments of `texture(ptlist, args)`.
pub struct TextureStyle<'a> {
    /// Number of texture strokes.
    pub tex: usize,
    pub wid: f64,
    /// Max half-length of a stroke, as a fraction of a contour.
    pub len: f64,
    /// Shade stroke width; also the stride between texture strokes, a
    /// quirk of the original kept because it controls the density.
    pub sha: usize,
    /// Noise amplitude for contour layer `l` (1-based).
    pub noi: &'a dyn Fn(f64) -> f64,
    /// Ink for the stroke at relative index `t ∈ [0, 1)`.
    pub col: &'a dyn Fn(&mut Chance, f64) -> Paint,
    /// Where along a contour a stroke is centred, in `[0, 1)`.
    pub dis: &'a dyn Fn(&mut Chance) -> f64,
}

fn default_noi(layer: f64) -> f64 {
    30.0 / layer
}

/// `rgba(100,100,100, rand*0.3)`.
pub fn faint_gray(chance: &mut Chance, _t: f64) -> Paint {
    Paint::ink(100, chance.random() * 0.3)
}

/// Either side of a contour, rarely its middle.
fn default_dis(chance: &mut Chance) -> f64 {
    if chance.random() > 0.5 {
        chance.random() / 3.0
    } else {
        2.0 / 3.0 + chance.random() / 3.0
    }
}

impl Default for TextureStyle<'_> {
    fn default() -> Self {
        Self {
            tex: 400,
            wid: 1.5,
            len: 0.2,
            sha: 0,
            noi: &default_noi,
            col: &faint_gray,
            dis: &default_dis,
        }
    }
}

/// Draws the texture of `layers`. Example: see `scenery::mount::mountain`.
pub fn texture(sketch: &mut Sketch, chance: &mut Chance, layers: &[Vec<Pt>], style: &TextureStyle) {
    if layers.len() < 2 || layers[0].is_empty() {
        return;
    }
    let lines: Vec<Vec<Pt>> = (0..style.tex)
        .map(|i| texture_line(chance, layers, style, i))
        .collect();
    if style.sha > 0 {
        shade(sketch, chance, &lines, style.sha);
    }
    let count = lines.len() as f64;
    for (j, line) in lines
        .iter()
        .enumerate()
        .skip(style.sha)
        .step_by(1 + style.sha)
    {
        let col = (style.col)(chance, j as f64 / count);
        let ss = StrokeStyle {
            col,
            wid: style.wid,
            ..Default::default()
        };
        stroke(sketch, chance, line, &ss);
    }
}

fn shade(sketch: &mut Sketch, chance: &mut Chance, lines: &[Vec<Pt>], sha: usize) {
    let ss = StrokeStyle {
        col: Paint::ink(100, 0.1),
        wid: sha as f64,
        ..Default::default()
    };
    for line in lines.iter().step_by(2) {
        stroke(sketch, chance, line, &ss);
    }
}

/// The i-th texture stroke: a run of points interpolated between two
/// neighbouring contours, jittered by noise.
fn texture_line(
    chance: &mut Chance,
    layers: &[Vec<Pt>],
    style: &TextureStyle,
    i: usize,
) -> Vec<Pt> {
    let cols = layers[0].len() as f64;
    let mid = ((style.dis)(chance) * cols) as isize;
    let hlen = (chance.random() * cols * style.len).floor() as isize;
    let clamp = |v: isize| v.clamp(0, cols as isize) as usize;
    let (start, end) = (clamp(mid - hlen), clamp(mid + hlen));
    let layer = i as f64 / style.tex as f64 * (layers.len() - 1) as f64;
    let amp = (style.noi)(layer + 1.0);
    (start..end)
        .map(|j| {
            let p = layer.fract();
            let [x, y] = between(
                &layers[layer.floor() as usize][j],
                &layers[layer.ceil() as usize][j],
                p,
            );
            let jn = j as f64 * 0.5;
            [
                x + amp * (chance.noise2(x, jn) - 0.5),
                y + amp * (chance.noise2(y, jn) - 0.5),
            ]
        })
        .collect()
}

/// The original weights the *floor* layer by `p` (not `1 - p`); kept, as it
/// is part of the look.
fn between(a: &Pt, b: &Pt, p: f64) -> Pt {
    [a[0] * p + b[0] * (1.0 - p), a[1] * p + b[1] * (1.0 - p)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh() -> Vec<Vec<Pt>> {
        (0..5)
            .map(|i| (0..20).map(|j| [j as f64 * 5.0, i as f64 * 10.0]).collect())
            .collect()
    }

    #[test]
    fn draws_up_to_tex_strokes() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        let style = TextureStyle {
            tex: 50,
            ..Default::default()
        };
        texture(&mut s, &mut c, &mesh(), &style);
        assert!(s.len() <= 50 && s.len() > 10, "drew {}", s.len());
    }

    #[test]
    fn shading_adds_strokes_and_strides() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        let style = TextureStyle {
            tex: 60,
            sha: 2,
            ..Default::default()
        };
        texture(&mut s, &mut c, &mesh(), &style);
        assert!(s.len() <= 30 + 20, "drew {}", s.len());
    }

    #[test]
    fn degenerate_mesh_is_ignored() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(3));
        texture(
            &mut s,
            &mut c,
            &[vec![[0.0, 0.0]]],
            &TextureStyle::default(),
        );
        assert!(s.is_empty());
    }

    #[test]
    fn between_weights_like_original() {
        assert_eq!(between(&[0.0, 0.0], &[10.0, 10.0], 0.25), [7.5, 7.5]);
    }
}

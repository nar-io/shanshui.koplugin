//! `water`: a few faint ripple lines under a mountain.

use crate::brush::{StrokeStyle, stroke};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::ink::{Paint, Sketch};

const CLU: usize = 10;
const LEN: f64 = 800.0;
const HEI: f64 = 2.0;

/// Example: `water(&mut sk, &mut ch, x, y)`.
pub fn water(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64) {
    let mut yk = 0.0;
    let ripples: Vec<Vec<Pt>> = (0..CLU)
        .map(|_| {
            let xk = (chance.random() - 0.5) * (LEN / 8.0);
            yk += chance.random() * 5.0;
            let lk = LEN / 4.0 + chance.random() * (LEN / 4.0);
            ripple(chance, xk, yk, lk)
        })
        .collect();
    // The original skips the first ripple when drawing.
    for line in &ripples[1..] {
        let style = StrokeStyle {
            col: Paint::ink(100, 0.3 + chance.random() * 0.3),
            wid: 1.0,
            ..Default::default()
        };
        stroke(sketch, chance, &translated(line, xoff, yoff), &style);
    }
}

fn ripple(chance: &Chance, xk: f64, yk: f64, lk: f64) -> Vec<Pt> {
    let mut out = Vec::new();
    let mut j = -lk;
    while j < lk {
        out.push([
            j + xk,
            (j * 0.2).sin() * HEI * chance.noise1(j * 0.1) - 20.0 + yk,
        ]);
        j += 5.0;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_ripples() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        water(&mut s, &mut c, 0.0, 500.0);
        assert_eq!(s.len(), CLU - 1);
    }

    #[test]
    fn ripple_spans_both_sides() {
        let r = ripple(&Chance::from_seed(2), 0.0, 0.0, 100.0);
        assert_eq!(r.len(), 40);
        assert!(r[0][0] < 0.0 && r[39][0] > 0.0);
    }
}

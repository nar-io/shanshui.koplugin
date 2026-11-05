//! `bezmh`: chains rational quadratic Bézier pieces through the midpoints of
//! a control polyline (weight `w` pulls towards the control points). Used
//! for the people's limbs and clothes.

use super::point::{Pt, mid_pt};

const STEPS: usize = 20;

/// Example: `bezmh(&[a, b, c], 2.0)` gives a smooth curve from `a` to `c`.
pub fn bezmh(ctrl: &[Pt], w: f64) -> Vec<Pt> {
    let ctrl: Vec<Pt> = match ctrl {
        [a, b] => vec![*a, mid_pt(&[*a, *b]), *b],
        _ => ctrl.to_vec(),
    };
    if ctrl.len() < 3 {
        return ctrl;
    }
    let pieces = ctrl.len() - 2;
    (0..pieces)
        .flat_map(|j| piece(&ctrl, j, w, j == pieces - 1))
        .collect()
}

fn piece(ctrl: &[Pt], j: usize, w: f64, last: bool) -> Vec<Pt> {
    let p0 = if j == 0 {
        ctrl[0]
    } else {
        mid_pt(&[ctrl[j], ctrl[j + 1]])
    };
    let p1 = ctrl[j + 1];
    let p2 = if last {
        ctrl[j + 2]
    } else {
        mid_pt(&[ctrl[j + 1], ctrl[j + 2]])
    };
    let count = STEPS + usize::from(last);
    (0..count)
        .map(|i| rational_quad(p0, p1, p2, w, i as f64 / STEPS as f64))
        .collect()
}

fn rational_quad(p0: Pt, p1: Pt, p2: Pt, w: f64, t: f64) -> Pt {
    let (a, b, c) = ((1.0 - t).powi(2), 2.0 * t * (1.0 - t) * w, t * t);
    let u = a + b + c;
    [
        (a * p0[0] + b * p1[0] + c * p2[0]) / u,
        (a * p0[1] + b * p1[1] + c * p2[1]) / u,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_and_ends_on_endpoints() {
        let c = bezmh(&[[0.0, 0.0], [5.0, 10.0], [10.0, 0.0]], 1.0);
        assert_eq!(c[0], [0.0, 0.0]);
        let end = c[c.len() - 1];
        assert!((end[0] - 10.0).abs() < 1e-9 && end[1].abs() < 1e-9);
        assert_eq!(c.len(), STEPS + 1);
    }

    #[test]
    fn two_points_become_line() {
        let c = bezmh(&[[0.0, 0.0], [10.0, 0.0]], 2.0);
        assert!(c.iter().all(|p| p[1].abs() < 1e-9));
    }

    #[test]
    fn four_points_two_pieces() {
        let c = bezmh(&[[0.0, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, 1.0]], 1.0);
        assert_eq!(c.len(), STEPS * 2 + 1);
    }
}

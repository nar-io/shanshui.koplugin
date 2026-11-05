//! `loopNoise` from the `#Util` script: makes a noise sequence start and end
//! on the same value (so closed blobs and rocks have no seam), then
//! normalises it to `[0, 1]`.

/// Example: `loop_noise(&mut [0.2, 0.5, 0.9])` leaves `[1.0, 0.0, ~0.6]`-ish values.
pub fn loop_noise(values: &mut [f64]) {
    let n = values.len();
    if n < 2 {
        return;
    }
    let dif = values[n - 1] - values[0];
    let (mut lo, mut hi) = (100.0_f64, -100.0_f64);
    for (i, v) in values.iter_mut().enumerate() {
        *v += dif * (n - 1 - i) as f64 / (n - 1) as f64;
        lo = lo.min(*v);
        hi = hi.max(*v);
    }
    let span = if hi > lo { hi - lo } else { 1.0 };
    values.iter_mut().for_each(|v| *v = (*v - lo) / span);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ends_meet_and_range_is_unit() {
        let mut v = vec![0.1, 0.4, 0.3, 0.8, 0.6];
        loop_noise(&mut v);
        assert!((v[0] - v[4]).abs() < 1e-12);
        let max = v.iter().cloned().fold(f64::MIN, f64::max);
        let min = v.iter().cloned().fold(f64::MAX, f64::min);
        assert!((max - 1.0).abs() < 1e-12 && min.abs() < 1e-12);
    }

    #[test]
    fn short_input_untouched() {
        let mut v = vec![0.3];
        loop_noise(&mut v);
        assert_eq!(v, vec![0.3]);
    }
}

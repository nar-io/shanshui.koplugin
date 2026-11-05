//! Binary coverage of a shape on the supersampled grid. A polygon (or all
//! the segment quads of an outline) is first unioned into a mask, then
//! composited once — so overlapping parts do not double their alpha.

use crate::geom::Pt;

/// Rectangular mask in supersampled pixel coordinates.
#[derive(Debug, Default)]
pub struct Coverage {
    pub x0: i64,
    pub y0: i64,
    pub w: usize,
    pub h: usize,
    pub bits: Vec<bool>,
    /// Scratch scanline crossings, kept to avoid an allocation per fill.
    crossings: Vec<(f64, i32)>,
}

impl Coverage {
    /// Resets to an empty mask over `[x0, x0+w) × [y0, y0+h)`, reusing memory.
    pub fn reset(&mut self, x0: i64, y0: i64, w: usize, h: usize) {
        (self.x0, self.y0, self.w, self.h) = (x0, y0, w, h);
        self.bits.clear();
        self.bits.resize(w * h, false);
    }

    /// Unions a nonzero-winding fill of `pts` (pixel coordinates; pixel
    /// centres sit at `.5`). Example: `cov.fill(&[[0.,0.],[4.,0.],[0.,4.]])`.
    pub fn fill(&mut self, pts: &[Pt]) {
        if pts.len() < 3 {
            return;
        }
        let (lo, hi) = pts.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| {
            (lo.min(p[1]), hi.max(p[1]))
        });
        let first = ((lo - 0.5).floor() as i64 - self.y0).max(0) as usize;
        let end = ((hi + 0.5).ceil() as i64 - self.y0).clamp(0, self.h as i64) as usize;
        let mut crossings = std::mem::take(&mut self.crossings);
        for row in first..end {
            let y = (self.y0 + row as i64) as f64 + 0.5;
            collect_crossings(pts, y, &mut crossings);
            self.fill_spans(row, &crossings);
        }
        self.crossings = crossings;
    }

    fn fill_spans(&mut self, row: usize, crossings: &[(f64, i32)]) {
        let mut winding = 0;
        for pair in crossings.windows(2) {
            winding += pair[0].1;
            if winding != 0 {
                self.fill_row(row, pair[0].0, pair[1].0);
            }
        }
    }

    /// Sets pixels of `row` whose centre lies in `[xa, xb)`.
    fn fill_row(&mut self, row: usize, xa: f64, xb: f64) {
        let first = ((xa - 0.5).ceil() as i64 - self.x0).max(0);
        let end = ((xb - 0.5).ceil() as i64 - self.x0).min(self.w as i64);
        if first >= end {
            return;
        }
        let base = row * self.w;
        self.bits[base + first as usize..base + end as usize].fill(true);
    }

    /// Unions a thick open polyline: one quad per segment, `width` wide.
    pub fn line(&mut self, pts: &[Pt], width: f64) {
        let half = (width / 2.0).max(0.5);
        for seg in pts.windows(2) {
            if let Some(quad) = segment_quad(seg[0], seg[1], half) {
                self.fill(&quad);
            }
        }
    }

    /// Iterates covered pixels as absolute `(x, y)`.
    pub fn covered(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        self.bits
            .iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| (self.x0 + (i % self.w) as i64, self.y0 + (i / self.w) as i64))
    }
}

/// Sorted x crossings of the scanline `y` with winding direction.
fn collect_crossings(pts: &[Pt], y: f64, out: &mut Vec<(f64, i32)>) {
    out.clear();
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let dir = if a[1] <= y && y < b[1] {
            1
        } else if b[1] <= y && y < a[1] {
            -1
        } else {
            0
        };
        if dir != 0 {
            let x = a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
            out.push((x, dir));
        }
    }
    out.sort_by(|p, q| p.0.total_cmp(&q.0));
}

fn segment_quad(a: Pt, b: Pt, half: f64) -> Option<[Pt; 4]> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-9 {
        return None;
    }
    // Extend along the segment too, so consecutive quads overlap at joints.
    let (ux, uy) = (dx / len * half, dy / len * half);
    let (nx, ny) = (-uy, ux);
    Some([
        [a[0] - ux + nx, a[1] - uy + ny],
        [b[0] + ux + nx, b[1] + uy + ny],
        [b[0] + ux - nx, b[1] + uy - ny],
        [a[0] - ux - nx, a[1] - uy - ny],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(c: &Coverage) -> usize {
        c.bits.iter().filter(|b| **b).count()
    }

    #[test]
    fn square_covers_its_area() {
        let mut c = Coverage::default();
        c.reset(0, 0, 10, 10);
        c.fill(&[[2.0, 2.0], [6.0, 2.0], [6.0, 6.0], [2.0, 6.0]]);
        assert_eq!(count(&c), 16);
    }

    #[test]
    fn nonzero_fills_overlapping_loops() {
        let mut c = Coverage::default();
        c.reset(0, 0, 10, 10);
        let sq = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let twice: Vec<Pt> = sq.iter().chain(sq.iter()).copied().collect();
        c.fill(&twice);
        assert_eq!(count(&c), 16);
    }

    #[test]
    fn clipped_to_mask() {
        let mut c = Coverage::default();
        c.reset(5, 5, 2, 2);
        c.fill(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]]);
        assert_eq!(count(&c), 4);
        assert_eq!(c.covered().next(), Some((5, 5)));
    }

    #[test]
    fn line_has_width() {
        let mut c = Coverage::default();
        c.reset(0, 0, 20, 10);
        c.line(&[[2.0, 5.0], [18.0, 5.0]], 2.0);
        let rows: std::collections::BTreeSet<i64> = c.covered().map(|p| p.1).collect();
        assert_eq!(rows.into_iter().collect::<Vec<_>>(), vec![4, 5]);
    }

    #[test]
    fn degenerate_segment_skipped() {
        assert!(segment_quad([1.0, 1.0], [1.0, 1.0], 1.0).is_none());
    }
}

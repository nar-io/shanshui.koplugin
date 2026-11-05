//! Building parts: `hut` (thatched cone), `box` (walls), `deco` (window
//! lattices) — the pieces `arch01..04` stack up.

use super::beams;
use crate::brush::{TextureStyle, texture};
use crate::chance::Chance;
use crate::geom::Pt;
use crate::geom::point::translated;
use crate::geom::subdivide::subdivide;
use crate::ink::{Paint, PolyStyle, Sketch};

/// Thatched roof cone hanging below `(xoff, yoff)`.
/// Example: `hut(&mut sk, &mut ch, x, y - 70.0, 30.0, 180.0)`.
pub fn hut(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, hei: f64, wid: f64) {
    let mesh = hut_mesh(chance, hei, wid);
    let (first, last) = (&mesh[0], &mesh[mesh.len() - 1]);
    let fill: Vec<Pt> = first[..first.len() - 1]
        .iter()
        .chain(last[..last.len() - 1].iter().rev())
        .copied()
        .collect();
    sketch.poly(translated(&fill, xoff, yoff), PolyStyle::paper());
    for edge in [first, last] {
        sketch.poly(
            translated(edge, xoff, yoff),
            PolyStyle::outlined(Paint::ink(100, 0.3), 2.0),
        );
    }
    let col = |ch: &mut Chance, _: f64| Paint::ink(120, 0.3 + ch.random() * 0.3);
    let dis = |ch: &mut Chance| ch.weighted(|a| a * a);
    let noi = |_: f64| 5.0;
    let style = TextureStyle {
        tex: 300,
        wid: 1.0,
        len: 0.25,
        col: &col,
        dis: &dis,
        noi: &noi,
        ..Default::default()
    };
    let shifted: Vec<Vec<Pt>> = mesh.iter().map(|row| translated(row, xoff, yoff)).collect();
    texture(sketch, chance, &shifted, &style);
}

/// Ten fan lines from the apex down to the eaves.
fn hut_mesh(chance: &mut Chance, hei: f64, wid: f64) -> Vec<Vec<Pt>> {
    (0..10)
        .map(|i| {
            let heir = hei + hei * 0.2 * chance.random();
            (0..10)
                .map(|j| {
                    let (fi, fj) = (i as f64 / 9.0, j as f64 / 9.0);
                    [wid * (fi - 0.5) * fj.powf(0.7), heir * fj]
                })
                .collect()
        })
        .collect()
}

/// The four corners a `deco` lattice is laid between.
#[derive(Clone, Copy, Debug)]
pub struct Facade {
    pub pul: Pt,
    pub pur: Pt,
    pub pdl: Pt,
    pub pdr: Pt,
}

/// Arguments of `box(x, y, args)`.
#[derive(Clone, Copy, Debug)]
pub struct BoxArgs {
    pub hei: f64,
    pub wid: f64,
    pub rot: f64,
    pub per: f64,
    /// Transparent: draw hidden edges and no paper fill.
    pub tra: bool,
    pub bot: bool,
    pub wei: f64,
    /// Window lattice style and spacing (`None` = plain wall).
    pub dec: Option<(u8, [usize; 2], [usize; 2])>,
}

impl Default for BoxArgs {
    fn default() -> Self {
        Self {
            hei: 20.0,
            wid: 120.0,
            rot: 0.7,
            per: 4.0,
            tra: true,
            bot: true,
            wei: 3.0,
            dec: None,
        }
    }
}

/// A walled storey seen at an angle. Example: `boxy(&mut sk, &mut ch, x, y, &BoxArgs::default())`.
pub fn boxy(sketch: &mut Sketch, chance: &mut Chance, xoff: f64, yoff: f64, a: &BoxArgs) {
    let (hw, mid, bmid) = (
        a.wid * 0.5,
        -a.wid * 0.5 + a.wid * a.rot,
        -a.wid * 0.5 + a.wid * (1.0 - a.rot),
    );
    let mut lines = vec![[[-hw, -a.hei], [-hw, 0.0]], [[hw, -a.hei], [hw, 0.0]]];
    if a.bot {
        lines.extend([[[-hw, 0.0], [mid, a.per]], [[hw, 0.0], [mid, a.per]]]);
    }
    lines.push([[mid, -a.hei], [mid, a.per]]);
    if a.tra && a.bot {
        lines.extend([[[-hw, 0.0], [bmid, -a.per]], [[hw, 0.0], [bmid, -a.per]]]);
    }
    if a.tra {
        lines.push([[bmid, -a.hei], [bmid, -a.per]]);
    }
    let mut polylines: Vec<Vec<Pt>> = lines.iter().map(|l| subdivide(l, 5)).collect();
    if let Some((style, hsp, vsp)) = a.dec {
        let surf = if a.rot < 0.5 { 1.0 } else { -1.0 };
        let face = Facade {
            pul: [surf * hw, -a.hei],
            pur: [mid, -a.hei + a.per],
            pdl: [surf * hw, 0.0],
            pdr: [mid, a.per],
        };
        polylines.extend(deco(style, &face, hsp, vsp));
    }
    if !a.tra {
        let outline = vec![
            [-hw, -a.hei],
            [hw, -a.hei],
            [hw, 0.0],
            [mid, a.per],
            [-hw, 0.0],
        ];
        sketch.poly(translated(&outline, xoff, yoff), PolyStyle::paper());
    }
    beams(sketch, chance, &polylines, [xoff, yoff], a.wei, 0.4);
}

/// Window lattice: 1 = `-| |-`, 2 = `||||`, 3 = `|##|`.
pub fn deco(style: u8, f: &Facade, hsp: [usize; 2], vsp: [usize; 2]) -> Vec<Vec<Pt>> {
    let dl = subdivide(&[f.pul, f.pdl], vsp[1]);
    let dr = subdivide(&[f.pur, f.pdr], vsp[1]);
    let du = subdivide(&[f.pul, f.pur], hsp[1]);
    let dd = subdivide(&[f.pdl, f.pdr], hsp[1]);
    let line = |a: Pt, b: Pt| subdivide(&[a, b], 5);
    if style == 2 {
        return (hsp[0]..du.len().saturating_sub(hsp[0]))
            .step_by(hsp[0].max(1))
            .map(|i| line(du[i], dd[i]))
            .collect();
    }
    let (mlu, mru) = (du[hsp[0]], du[du.len() - 1 - hsp[0]]);
    let (mld, mrd) = (dd[hsp[0]], dd[du.len() - 1 - hsp[0]]);
    let mut out = Vec::new();
    for i in (vsp[0]..dl.len().saturating_sub(vsp[0])).step_by(vsp[0].max(1)) {
        let mml = subdivide(&[mlu, mld], vsp[1])[i];
        let mmr = subdivide(&[mru, mrd], vsp[1])[i];
        if style == 1 {
            out.extend([line(mml, dl[i]), line(mmr, dr[i])]);
        } else {
            let mmu = subdivide(&[mlu, mru], vsp[1])[i];
            let mmd = subdivide(&[mld, mrd], vsp[1])[i];
            out.extend([line(mml, mmr), line(mmu, mmd)]);
        }
    }
    out.extend([line(mlu, mld), line(mru, mrd)]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FACE: Facade = Facade {
        pul: [0.0, 0.0],
        pur: [0.0, 100.0],
        pdl: [100.0, 0.0],
        pdr: [100.0, 100.0],
    };

    #[test]
    fn deco_styles_produce_lines() {
        assert_eq!(deco(2, &FACE, [1, 5], [1, 2]).len(), 4);
        assert_eq!(deco(1, &FACE, [1, 5], [1, 2]).len(), 2 + 2);
        assert_eq!(deco(3, &FACE, [1, 4], [1, 3]).len(), 4 + 2);
    }

    #[test]
    fn opaque_box_has_paper() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(1));
        boxy(
            &mut s,
            &mut c,
            0.0,
            0.0,
            &BoxArgs {
                tra: false,
                ..Default::default()
            },
        );
        assert_eq!(s.polygons[0].style.fill, Paint::Paper);
    }

    #[test]
    fn hut_draws() {
        let (mut s, mut c) = (Sketch::new(), Chance::from_seed(2));
        hut(&mut s, &mut c, 0.0, 0.0, 40.0, 180.0);
        assert!(s.len() > 50);
    }
}

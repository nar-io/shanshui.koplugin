use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use shanshui_rs::ink::svg::to_svg;
use shanshui_rs::ink::{InkPolygon, Sketch};
use shanshui_rs::world::World;

fn render_svg_to_png_resvg(
    svg_str: &str,
    width: u32,
    height: u32,
    is_transparent_sky: bool,
    out_path: &Path,
) -> Result<(), String> {
    let mut opt = resvg::usvg::Options::default();
    opt.font_family = "sans-serif".to_string();

    let tree = resvg::usvg::Tree::from_str(svg_str, &opt)
        .map_err(|e| format!("SVG parse error: {e}"))?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to allocate pixmap".to_string())?;

    // Always render on pure white so mountains, rivers, valleys, paper-fills remain opaque white
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    let tree_w = tree.size().width();
    let tree_h = tree.size().height();
    let sx = width as f32 / tree_w;
    let sy = height as f32 / tree_h;
    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let file = File::create(out_path)
        .map_err(|e| format!("cannot create {}: {e}", out_path.display()))?;
    let mut enc = png::Encoder::new(BufWriter::new(file), width, height);

    let rgba_data = pixmap.data();
    let w = width as usize;
    let h = height as usize;

    if is_transparent_sky {
        // Step 1: Detect the mountain ridge skyline across every column X.
        // Sky is strictly the area above the uppermost mountain ridge.
        // Everything below (valleys, mist, rivers, pagodas, paper-fills) remains 100% opaque.
        let mut mountain_top = vec![h; w];
        for x in 0..w {
            for y in 0..h {
                let idx = y * w + x;
                let px = &rgba_data[idx * 4..idx * 4 + 4];
                let r = px[0] as f32;
                let g = px[1] as f32;
                let b = px[2] as f32;
                let gray = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
                // If any ink stroke appears
                if gray < 242 {
                    mountain_top[x] = y;
                    break;
                }
            }
        }

        // Step 2: Smooth ridge peaks so sky follows a natural continuous skyline
        let mut smoothed_top = mountain_top.clone();
        for x in 1..w - 1 {
            let left = mountain_top[x - 1];
            let mid = mountain_top[x];
            let right = mountain_top[x + 1];
            smoothed_top[x] = mid.min((left + right) / 2 + 1);
        }

        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc
            .write_header()
            .map_err(|e| format!("png header: {e}"))?;

        let mut out_buf = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let px = &rgba_data[idx * 4..idx * 4 + 4];
                let r = px[0] as f32;
                let g = px[1] as f32;
                let b = px[2] as f32;
                let gray = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;

                let peak_y = smoothed_top[x];
                let alpha = if y < peak_y {
                    if peak_y > 3 && y >= peak_y - 3 {
                        // Feathered edge to mountain summit
                        let factor = (y - (peak_y - 3)) as f32 / 3.0;
                        (factor * 255.0).round() as u8
                    } else {
                        0u8 // Truly transparent sky!
                    }
                } else {
                    255u8 // 100% opaque landscape, valley, river, and mountain body
                };

                out_buf.push(gray);
                out_buf.push(gray);
                out_buf.push(gray);
                out_buf.push(alpha);
            }
        }
        writer
            .write_image_data(&out_buf)
            .map_err(|e| format!("png write: {e}"))
    } else {
        // Pure 8-bit grayscale without transparency
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc
            .write_header()
            .map_err(|e| format!("png header: {e}"))?;

        let mut gray_buf = Vec::with_capacity(w * h);
        for chunk in rgba_data.chunks_exact(4) {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;
            let gray = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
            gray_buf.push(gray);
        }
        writer
            .write_image_data(&gray_buf)
            .map_err(|e| format!("png write: {e}"))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: shanshui_rs <output.png|output.svg> [seed] [width] [height] [--transparent]\n\
             Example:\n  shanshui_rs current.png 1234 1236 1648 --transparent"
        );
        std::process::exit(1);
    }

    let out_path = std::path::PathBuf::from(&args[1]);
    let is_transparent = args.iter().any(|a| a == "--transparent");

    let seed = if args.len() > 2 && !args[2].starts_with("--") {
        args[2].clone()
    } else {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        ms.to_string()
    };

    let width: usize = if args.len() > 3 && !args[3].starts_with("--") {
        args[3].parse().unwrap_or(1236)
    } else {
        1236
    };

    let height: usize = if args.len() > 4 && !args[4].starts_with("--") {
        args[4].parse().unwrap_or(1648)
    } else {
        1648
    };

    let is_svg = out_path
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("svg"))
        .unwrap_or(false);

    let t0 = Instant::now();

    let is_portrait = height >= width;
    let aspect = width as f64 / height as f64;

    let (v_width, v_height, min_y, min_x) = if is_portrait {
        let vw = 580.0;
        let vh = (vw / aspect).round();
        (vw, vh, -35.0, 0.0)
    } else {
        let vh = 650.0;
        let vw = (vh * aspect).round();
        (vw, vh, -20.0, 0.0)
    };

    let xmin = min_x;
    let xmax = min_x + v_width;

    // 1. Generate Shan Shui world chunks
    let mut world = World::new(&seed);
    for k in world.missing(xmin, xmax) {
        world.generate(k);
    }

    let polygons: Vec<InkPolygon> = world.polygons_in(xmin, xmax);
    let t_gen = Instant::now();

    let mut sketch = Sketch::new();
    sketch.polygons = polygons;
    let svg_data = to_svg(
        &sketch,
        [min_x, min_y, v_width, v_height],
        width,
        height,
        false,
    );

    if is_svg {
        std::fs::write(&out_path, svg_data)
            .unwrap_or_else(|e| eprintln!("Failed to write SVG: {e}"));
        let t_end = Instant::now();
        println!(
            "ShanShui (Rust): Generated SVG in {:?} (write {:?}) -> {} (seed={})",
            t_gen.duration_since(t0),
            t_end.duration_since(t_gen),
            out_path.display(),
            seed
        );
    } else {
        if let Err(e) = render_svg_to_png_resvg(
            &svg_data,
            width as u32,
            height as u32,
            is_transparent,
            &out_path,
        ) {
            eprintln!("Failed to render PNG via resvg: {e}");
            std::process::exit(2);
        }
        let t_end = Instant::now();
        println!(
            "ShanShui (Rust): Gen {:?} + Resvg Render {:?} = Total {:?} -> {} ({}x{}, seed={}, transparent_sky={})",
            t_gen.duration_since(t0),
            t_end.duration_since(t_gen),
            t_end.duration_since(t0),
            out_path.display(),
            width,
            height,
            seed,
            is_transparent
        );
    }
}

# `shanshui_rs`: Procedural Chinese Landscape Engine

A fast, standalone Rust implementation of LingDong Huang's [{Shan, Shui}*](https://github.com/LingDong-/shan-shui-inf) procedural landscape generator.

This engine compiles into a self-contained static ELF binary with zero runtime dependencies, embedding procedural geometry generation, Chinese ink brush stroke algorithms, and a native Skia-based rasterizer (`tiny-skia` / `resvg`).

---

## 🏗️ Architecture & Features

1. **Noise & Procedural Math:**
   - Perlin noise and fractional Brownian motion (fBm) generators.
   - Algorithmic Chinese brush stroke synthesizers: arches, strokes, mountain outlines, textured fills, tree foliage, and pagoda structures.

2. **Embedded Rasterization:**
   - Generates procedural vector strokes into memory and renders directly via `resvg` / `tiny-skia`.
   - Produces pixel-perfect PNGs directly on embedded devices without needing external graphics runtimes.

3. **Skyline Silhouette Algorithm (`--transparent`):**
   - Implements a column-based raycast and flood traversal to determine the highest mountain ridge across all columns.
   - Modifies pixels strictly above the ridge line (`y < ridge_y`) to transparent alpha (`A = 0`) with 3-pixel anti-aliased edge feathering.
   - Preserves valleys, rivers, and internal mountain paper textures at 100% opaque white (`A = 255`).

---

## 🛠️ CLI Usage

```sh
shanshui_rs <output_png> [seed] [width] [height] [--transparent]
```

### Examples:
- Default generation:
  ```sh
  shanshui_rs landscape.png 12345 1236 1648
  ```
- Transparent sky generation:
  ```sh
  shanshui_rs landscape.png 12345 1236 1648 --transparent
  ```

---

## 📦 Compilation

```sh
# Modern Kindles (ARMv7 hard-float)
cargo build --release --target armv7-unknown-linux-musleabihf

# Legacy Kindles (ARMv6/v7 soft-float)
cargo build --release --target arm-unknown-linux-musleabi

# Modern 64-bit e-readers (AArch64)
cargo build --release --target aarch64-unknown-linux-musl

# Host build
cargo build --release
```

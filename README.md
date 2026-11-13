# Shan Shui Screensaver for KOReader (`shanshui.koplugin`)

A screensaver plugin for [KOReader](https://github.com/koreader/koreader) that generates infinite procedural Chinese landscape paintings (*Shanshui* 水墨画) every time your e-reader enters sleep mode.

Based on procedural landscape generation algorithms by [LingDong Huang](https://github.com/LingDong-/shan-shui-inf).

---

## 🌟 Features

- **Procedural Chinese Landscape Art:** Each landscape is uniquely generated from mathematical noise functions, featuring mountain ridges, trees, pagodas, and traditional ink brushwork.
- **High-Performance Native Engine:** Standalone native binary with embedded vector rasterization (`tiny-skia` / `resvg`), running directly on e-reader hardware in seconds with zero external dependencies.
- **Automatic Screen Resolution & Device Profile Detection:** Reads device screen dimensions directly from KOReader and applies the optimal canvas profile:
  - **1236 x 1648:** Kindle Paperwhite 5 (11th Gen) / Signature Edition
  - **1404 x 1872:** Kindle Oasis 2/3, Kobo Libra 2 / Colour, Clara 2E / BW, Tolino Vision 6
  - **1072 x 1448:** Kindle Paperwhite 3/4, Voyage, Oasis 1, Kobo Clara HD / Glo HD
  - **1860 x 2480:** Kindle Scribe, Kobo Elipsa 1/2E, Boox Note Air
  - **1264 x 1680:** Kobo Forma, Kobo Sage
  - **758 x 1024:** Kindle Paperwhite 1/2, Kobo Aura / Touch 2.0
  - **600 x 800:** Standard Fallback (Kindle Basic 4–10, Legacy Kobo / Sony Reader)
- **Selective Transparent Sky (Skyline Silhouette):**
  - Optional setting toggleable from the plugin menu.
  - Keeps mountain bodies, valleys, and river fills 100% opaque white while rendering **only the sky above the mountain ridges transparent**.
  - Allows current book covers or reading pages to be visible behind the mountain skyline during sleep mode.
- **Orientation Awareness:** Keeps one generated picture per screen formation (`portrait.png` and `landscape.png`), so the screensaver always prints the picture that matches the way the device is being held right now — rotating the screen and going straight to sleep no longer prints the previous formation. Follows the live rotation automatically, or can be locked to portrait/landscape.
- **Multi-Architecture Support:**
  - `shanshui_armhf`: ARMv7 with hardware VFP/NEON (Kindle PW3/4/5, Oasis, Voyage, modern Kobos).
  - `shanshui_armel`: ARMv6/v7 soft-float (Legacy Kindles: Kindle 3 Keyboard, K4, K5, Touch, PW1/2, Kindle Basic).
  - `shanshui_aarch64`: 64-bit ARM (Onyx Boox, modern Android e-readers, 64-bit Kobos).
  - Automatic architecture detection with seamless fallback.

---

## 📂 Installation

1. Copy or clone this repository into your device's KOReader plugins directory:
   ```sh
   # On Kindle:
   git clone https://github.com/nar-io/shanshui.koplugin.git /mnt/us/koreader/plugins/shanshui.koplugin
   ```
   *(Or download the zip release and extract the `shanshui.koplugin` folder to `koreader/plugins/`)*

2. Restart KOReader:
   - Tap top menu -> **Exit -> Restart KOReader**.

3. Configure the plugin:
   - Tap top menu -> **Tools (Wrench icon) -> Shan Shui Screensaver**.
   - Enable the screensaver, choose your resolution profile (or keep Auto-detect), and toggle Transparent Sky if desired.

---

## 🏗️ Building from Source

To compile the native landscape generation engine binaries from source:

```sh
cd shanshui_rs

# ARMv7 hard-float (Modern Kindles & Kobos)
cargo build --release --target armv7-unknown-linux-musleabihf

# ARMv6/v7 soft-float (Legacy Kindles)
cargo build --release --target arm-unknown-linux-musleabi

# AArch64 (64-bit ARM e-readers)
cargo build --release --target aarch64-unknown-linux-musl

# Host (Desktop Linux / Windows)
cargo build --release
```

Compiled binaries should be copied into the `bin/` directory:
- `shanshui_armhf`
- `shanshui_armel`
- `shanshui_aarch64`
- `shanshui_armv7`

---

## 📜 License

MIT License. Procedural landscape generation algorithms based on [LingDong-/shan-shui-inf](https://github.com/LingDong-/shan-shui-inf).

# Shan Shui Screensaver for KOReader

A screensaver for [KOReader](https://github.com/koreader/koreader) that paints a new Chinese ink landscape (*shan shui*, 山水) every time your e-reader goes to sleep. No two pictures are the same.

The idea comes from [LingDong Huang's shan-shui-inf](https://github.com/LingDong-/shan-shui-inf) — thanks to him for the original generator.

---

## What it does

- **A new landscape every sleep.** Mountains, mist, trees, pagodas and rivers are drawn fresh each time, with a hand-drawn ink look.
- **Fits your screen.** The plugin reads your device's resolution and picks the right canvas size automatically, so nothing is stretched or cropped.
- **Transparent sky (optional).** Turn it on and only the sky above the mountain ridges becomes see-through, so your book cover or reading page shows through while the mountains stay solid white. A nice effect on lock screen.
- **Follows the way you hold it.** It keeps a portrait and a landscape picture, and prints whichever matches how the device is held right now — even if you rotate and go straight to sleep.
- **Works on most e-readers.** Kindle, Kobo, Tolino and Onyx Boox are covered, old and new; the right version is picked automatically.
- **Fast.** Pictures are made by a small native program that runs on the device itself, in a couple of seconds, with nothing else installed.

---

## Installation

1. Copy the `shanshui.koplugin` folder into your KOReader `plugins` directory.

   On a Kindle that is usually `/mnt/us/koreader/plugins/`. You can also download the zip from the Releases page and unpack it there.

2. Restart KOReader: **Exit → Restart KOReader**.

3. Turn it on: **Tools (wrench icon) → Shan Shui Screensaver**. Enable the screensaver, and set the other options to taste.

---

## Settings

| Setting | What it does |
|---|---|
| Enable screensaver | Uses the generated picture when the device sleeps |
| Screen profile | Auto-detect, or force a specific resolution |
| Transparent sky | Only the sky above the mountains becomes transparent |
| Orientation | Follow the device, or lock to portrait / landscape |

---

## Supported screens

| Resolution (portrait) | Devices |
|---|---|
| 1264 × 1680 | Kindle Paperwhite 12th gen (2024) and 2026, Paperwhite Signature Edition, Kindle Colorsoft (2024–2026), Kindle Oasis 2/3, Kobo Libra 2 / Colour, Kobo Clara 2E / Colour / BW, Tolino Vision 6, PocketBook Era |
| 1236 × 1648 | Kindle Paperwhite 5 / 11th gen (2021), Signature Edition, Kids |
| 1072 × 1448 | Kindle (2022 / 2024 / 2026), Kindle Paperwhite 3/4, Voyage, Oasis 1, Kobo Clara HD / Glo HD, Nook GlowLight 4 |
| 1980 × 2640 | Kindle Scribe 11" (2025 / 2026), Kindle Scribe Colorsoft |
| 1860 × 2480 | Kindle Scribe 10.2" (2022 / 2024) |
| 1404 × 1872 | Kobo Elipsa 1 / 2E, Onyx Boox Note Air, PocketBook InkPad 4 / Color 3 |
| 758 × 1024 | Kindle Paperwhite 1/2, Kobo Aura, Kobo Nia |
| 600 × 800 | Kindle Basic 4–10, Kindle 4/5/Touch/Keyboard, older Kobo and Sony readers (fallback) |

Anything not listed is detected from the real screen size and drawn at that size. If a device reports a size a few pixels off a known panel, the closest profile is used so the picture still fills the screen.

---

## Building it yourself

Only needed if you want to change the drawing engine. Prebuilt binaries are already in `bin/`.

```sh
cd shanshui_rs
cargo build --release
```

For an e-reader you need to cross-compile for its processor:

| Device type | Target |
|---|---|
| Modern Kindle and Kobo (ARMv7) | `armv7-unknown-linux-musleabihf` |
| Older Kindle (ARMv6/v7, soft float) | `arm-unknown-linux-musleabi` |
| 64-bit readers (Onyx Boox, newer Kobo) | `aarch64-unknown-linux-musl` |

Copy the resulting binary into `bin/` under the matching name: `shanshui_armhf`, `shanshui_armel`, `shanshui_aarch64` or `shanshui_armv7`.

---

## License

MIT. The landscape generation is based on [LingDong-/shan-shui-inf](https://github.com/LingDong-/shan-shui-inf).

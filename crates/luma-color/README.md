# gpui-luma-color

Color system primitives and interactive controls for **GPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides domain renderers, checkerboard painting, palette calculations, and interactive color manipulation components built on the lookless Luma SDK.

## Components

* **`ColorField`**: 2D saturation/brightness or hue/saturation selection surface.
* **`ColorRing` & `ColorArc`**: Radial hue and chromaticity selectors.
* **`ColorSlider`**: 1D gradient sliders for alpha, hue, saturation, and lightness.
* **`ColorSwatch` & `ColorSwatchButtonTemplate`**: Swatches with checkerboard transparency backdrops and selected states.
* **`CheckerboardPaint`**: GPU-friendly alpha background rendering.

## Installation

```sh
cargo add gpui-luma --rename luma --git https://github.com/scottcg/gpui-luma
cargo add gpui-luma-color --rename luma-color --git https://github.com/scottcg/gpui-luma
```

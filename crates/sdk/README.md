# gpui-luma

Lookless control SDK and interaction engine for **GPUI-Luma** ([gpui-luma.dev](https://gpui-luma.dev)).

Provides component logic, event routing, keyboard navigation, focus management, drag-and-drop, and theme contracts independent of any visual theme or styling adapter.

## Modules

* **`controls`**: Buttons, text inputs, checkboxes, radios, switches, sliders, tree views, list boxes, menus, and overlays.
* **`focus`**: Two-dimensional focus navigation, ring indicators, and directional traversal.
* **`interaction`**: Hover tracking, active press state, double-click detection, and pointer capture.
* **`layouts`**: Dock panels, grid systems, wide-middle splits, and layer stacks.
* **`motion`**: Transitions, overlay enter/exit scaling, and continuous phases.
* **`theme`**: Look-agnostic theme provenance and style resolution traits.

## Usage

Depend on `gpui-luma` directly and choose a look adapter for styled controls:

```sh
cargo add gpui-luma --rename luma --git https://github.com/scottcg/gpui-luma
```

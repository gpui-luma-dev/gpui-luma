# Radix Icons

SVG files in this directory are from [Radix Icons](https://www.radix-ui.com/icons)
(`@radix-ui/react-icons`), copyright WorkOS, licensed under the MIT License.

Regenerate the Rust catalog after replacing SVGs:

```sh
python3 scripts/generate_react_icon_catalog.py \
  --dir apps/radix-studio/src/assets/react-icons \
  --out apps/radix-studio/src/assets/react_icons/catalog.rs
```

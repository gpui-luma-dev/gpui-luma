# gpui-opal

Lookless GPUI toolkit split from the `gpui-lookless` spike into a dedicated workspace.

## Workspace

- `crates/sdk` - core lookless controls, behavior models, and look contracts.
- `apps/gallery` - interactive gallery app for controls and look packs.

## Run

```bash
cargo run -p gpui-opal-gallery
MTL_HUD_ENABLED=1 cargo run -p gpui-opal-gallery --release
```

## Notes

Migration notes from the spike are in `docs/notes/`.

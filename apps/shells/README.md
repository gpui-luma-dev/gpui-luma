# Shell demo apps

Full-window reference apps for each `SplitView` app-level shell recipe. Each app uses the gallery Properties `NavigationSidebar` sample, gallery-style theme CLI, and a title-bar light/dark toggle.

## Run

```bash
cargo run -p gpui-luma-shell-unified -- default
cargo run -p gpui-luma-shell-inset -- retro-arcade
cargo run -p gpui-luma-shell-icon-rail -- default
cargo run -p gpui-luma-shell-detached -- default
cargo run -p gpui-luma-shell-split-titlebar -- default
```

Pass `default` or any built-in shadcn theme stem (e.g. `jarvis`, `retro-arcade`) as the first CLI argument.

## Variants

| Package | Shell recipe |
|---|---|
| `gpui-luma-shell-unified` | Baseline sidebar + content split |
| `gpui-luma-shell-inset` | Layered inset frame around the split |
| `gpui-luma-shell-icon-rail` | Collapse to icon-rail width (128px) |
| `gpui-luma-shell-detached` | Independent nav/content surfaces; menu toggle in content |
| `gpui-luma-shell-split-titlebar` | Full-window `ResizablePanels` (title bar included in each pane) |

Shared code lives in `gpui-luma-shell-common`.

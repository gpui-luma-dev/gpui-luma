# Shell demo apps

Full-window reference apps for each `SplitView` app-level shell recipe. Each app uses the Properties `SidebarControl` sample from `apps/shells/common`, the shared shadcn theme CLI, and a title-bar light/dark toggle.

## Run

```bash
cargo run -p luma-shell-unified -- default
cargo run -p luma-shell-inset -- retro-arcade
cargo run -p luma-shell-icon-rail -- default
cargo run -p luma-shell-detached -- default
cargo run -p luma-shell-split-titlebar -- default
cargo run -p luma-shell-vscode -- default
```

Pass `default` or any built-in shadcn theme stem (e.g. `jarvis`, `retro-arcade`) as the first CLI argument.

## Variants

| Package | Shell recipe |
|---|---|
| `luma-shell-unified` | Baseline sidebar + content split |
| `luma-shell-inset` | Layered inset frame around the split |
| `luma-shell-icon-rail` | Collapse to icon-rail width (128px) |
| `luma-shell-detached` | Independent nav/content surfaces; menu toggle in content |
| `luma-shell-split-titlebar` | Full-window `ResizablePanels` (title bar included in each pane) |
| `luma-shell-vscode` | VS Code workbench shell: Customize Layout regions (Primary Side Bar, Editor, Status Bar) |

Shared code lives in `luma-shell-common`.

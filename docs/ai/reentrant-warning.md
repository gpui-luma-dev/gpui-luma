# GPUI Entity Re-entrancy

GPUI apps in this workspace (especially `apps/theme-studio`) use a tree of **entities** (`cx.new`, `Entity<T>`, `.update`, `.read`, `Render`). A common runtime failure is:

```text
cannot read ThemeStudioApp while it is already being updated
cannot update ThemeStudioApp while it is already being updated
```

This document explains the cause, the fix used in theme-studio, and guidelines for anyone extending these apps.

## What GPUI enforces

While an entity is inside `.update(cx, |…| { … })`, GPUI **leases** it: the value is moved to a stack frame and cannot be read or updated again until that closure returns. The same rule applies to **globals** during `update_global` (the global is removed from the map for the duration of the closure).

`cx.notify()` can schedule or trigger re-renders **before** the outer update returns. Any code in that re-render path must not touch a leased entity (or leased global).

This is a **runtime** check, not a compile-time guarantee.

## How theme-studio hit it

Typical chain when editing a token color in the sidebar:

1. `ThemeStudioApp::set_global_color` — app entity is **updating**
2. → `apply_theme_overrides` → `refresh_content_pane`
3. → `content_pane.update(…)` on `ContentPaneHost`
4. → palette sync / `cx.notify()` → `ContentPaneHost::render`
5. → **old code** called `self.app.read(cx)` to get look, layout, demos
6. **Panic** — `ThemeStudioApp` was still leased from step 1

A related failure mode: subscribing on `ThemeStudioApp` for tab events and calling `content_pane.update` from that handler re-entered the app entity (`cannot update … while already being updated`). See `ThemeSidebar::wire_subscriptions` for the inverse case (subscribe on the app, not the sidebar, because handlers update the sidebar).

## Solution: coordinator + snapshots

Theme-studio uses a **hybrid** pattern (not globals for view state — see [What we do not do](#what-we-do-not-do)).

### 1. One coordinator entity

`ThemeStudioApp` owns authoritative state: look, overrides, panel layout, selection, demos. Child hosts are views and event sinks.

### 2. Push snapshots down (read path)

Define plain-data structs suffixed with `Snapshot`. The child stores the latest snapshot and **never reads the parent in `render`**.

```text
ThemeStudioApp::board_snapshot()     // clone on coordinator
  → content_pane.update(…)
  → ContentPaneHost::sync_board_snapshot(board, cx)
  → Render uses self.board only
```

Reference: `BoardSnapshot` and `ContentPaneHost` in `apps/theme-studio/src/studio/content_pane.rs`.

### 3. Clone before `child.update`

The parent gathers all data **before** calling `child.update`. Never call `parent.read(cx)` inside the child’s update closure.

Reference: `ThemeStudioApp::refresh_content_pane` in `apps/theme-studio/src/studio/app.rs`.

### 4. Write path: update parent, then re-snapshot

When a child must mutate the coordinator (e.g. panel drag), call `app.update(…)` from an event handler, then refresh local cache **after** that closure returns:

```text
app.update(cx, |app, cx| { … });
pull_board_from_app(cx);   // app.read is safe here
```

Reference: `begin_panel_drag`, `handle_panel_drag_move`, `pull_board_from_app` in `content_pane.rs`.

### 5. Subscribe on the entity you mutate in the handler

`cx.subscribe` runs as an update on the **subscriber** entity. Do not call `subscriber.update` from that entity’s own subscription.

| Situation | Subscribe on |
|-----------|----------------|
| Token field changes → mutate app | `ThemeStudioApp` (`ThemeSidebar::wire_subscriptions`) |
| Tab activation → mutate content host | `ContentPaneHost` (tab subscription in `ContentPaneHost::new`) |

## Naming conventions

- **Struct:** `*Snapshot` (e.g. `BoardSnapshot`) — point-in-time, safe to read in render.
- **Method:** `*_snapshot()` on the coordinator; `sync_*_snapshot()` on the child.
- **Comment** at entity boundaries when the rule is non-obvious (see comments on `BoardSnapshot` and `wire_subscriptions`).

## Review checklist

Before merging GPUI UI in `apps/`:

1. Does any `Render::render` call `.read` on a parent or sibling entity?
2. Does any `child.update` closure call `parent.read`?
3. Does any subscription on entity `E` call `E.update`?
4. After child → parent writes, is parent data re-cached only **after** `parent.update` returns?
5. Is there one clear owner for each piece of mutable state?

## What we do not do

**Globals as primary app state.** `gpui::Global` is appropriate for app-wide infrastructure (e.g. `LumaLayoutCacheRegistry` in the SDK), not for per-window editor state (panel positions, selection, tab, demos). Globals have their own lease during `update_global`; reading the same global inside that closure (e.g. from a nested render) also panics. They do not remove the need for snapshot discipline.

**Defer as the default fix.** `cx.defer_in(window, …)` can break nested update chains but adds timing complexity. Prefer snapshots for display state and update-then-resnapshot for writes.

## Mental model

Same as familiar UI flows, with Rust-style exclusive access:

- **Props down** — snapshots pushed via `refresh_*` / `sync_*_snapshot`
- **Events up** — subscriptions on the coordinator (or the host, when only the host should change)
- **Render** — local / cached fields only; never reach up the entity tree

## Further reading in-repo

- `apps/theme-studio/src/studio/content_pane.rs` — `BoardSnapshot`, render without `app.read`
- `apps/theme-studio/src/studio/app.rs` — `board_snapshot`, `refresh_content_pane`
- `apps/theme-studio/src/studio/theme_sidebar.rs` — `wire_subscriptions` and subscription placement
- `docs/ai/architecture.md` — entity model and eventing overview

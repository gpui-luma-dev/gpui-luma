# GPUI-Luma Project Overview

## Scope

This repository is a Rust workspace for a GPUI-based component SDK and a gallery app that exercises the SDK.

- Workspace root: `gpui-luma/`
- Main crates:
  - `crates/sdk` → `gpui-luma` (library crate)
  - `apps/gallery` → `gpui-luma-gallery` (application crate)
  - `apps/theme-studio` → `gpui-luma-theme-studio` (theme customizer / styleguide app)

## Workspace Structure

- `Cargo.toml` (workspace)
  - members: `crates/sdk`, `apps/gallery`, `apps/theme-studio`
  - edition: Rust 2024
  - key shared deps: `gpui`, `gpui_platform`, `anyhow`, `lucide-icons`, `serde`, `tiny-skia`, `toml`
- `justfile`
  - `just gallery` / `just gallery-rel` to run the gallery app
  - `just theme-studio` to run the Theme Studio customizer
- `rust-toolchain.toml`
  - stable toolchain
  - includes `rustfmt`, `clippy`

## Crate Summaries

## `gpui-luma` (`crates/sdk`)

Purpose: reusable UI control SDK built on GPUI.

Top-level modules:

- `init` – global SDK init (`init(cx)`) currently registers Lucide font bytes
- `focus` – shared focus actions/context and focus-scope extension trait
- `keyhandling` – default keyboard action profiles + default key bindings
- `theme` – native tokens, Radix/CSS runtime theming, usage registry, interaction/adorner types
- `controls` – control library (buttons, inputs, menus, navigation, layout helpers)
- `shell` – shared shell components (`TitleBar`)

## `gpui-luma-gallery` (`apps/gallery`)

Purpose: showcase/demo app for SDK controls and theming.

High-level flow:

1. Build `gpui_platform::application().with_assets(Assets)`
2. Call `gpui_luma::init(cx)`
3. Register default focus and control key bindings
4. Open app window (`app_shell::open`) and mount `GalleryApp`

## Important Public APIs

## SDK bootstrap

- `gpui_luma::init(cx: &mut gpui::App) -> anyhow::Result<()>`
- `gpui_luma::focus::bind_default_focus_keys(&mut App)`
- `gpui_luma::keyhandling::bind_default_control_keys(&mut App)`

## Focus + keyboard semantics

- `focus::LumaFocusScopeExt::luma_focus_scope(...)`
- `keyhandling::ControlKeyProfile` and `default_control_key_bindings()`

## Theme APIs

- `theme::LumaTheme`, `theme::ThemeTokens`, `theme::ThemeMode` — native fallback schema
- `theme::RadixTheme`, `theme::set_active_radix_theme` — CSS-first product runtime theme
- `theme::all_radix_theme_usages` — CSS token usage metadata for gallery introspection
- `theme::LumaChrome` — shell chrome colors
- `theme::ThemeUsage`, `theme::ThemePartUsage`
- `theme::palette_color_tokens(...)`, `theme::resolve_palette_color(...)`

## Controls API style

Common pattern across controls:

- `new(id)` constructor returning a builder (`*Builder`)
- `.template(...)` + fluent model options
- `.spawn(cx)` to create GPUI entity
- control event enum (`*Event`) emitted by entity

Frequently used controls:

- command family: `command::button`, `command::icon_button`, plus `checkbox`, `radio_button`, `switch`, `toggle`
- text inputs: `textfield`, `textarea`
- list/select: `selector`, `autocomplete`, `combobox` (composite selection: `control_group`, `radio_group`, `button_group`)
- menus: `popup_menu`, `context_menu`, `floating_menu`
- navigation/layout: `navigation_sidebar`, `tabs_navigation`, `split_view`, `scrollbar`, `slider`, `progress`

## Core Data Types and Conventions

- Shared interaction primitives:
  - `theme::InteractionState`
  - `controls::state::ControlFocusState`
  - `controls::state::CompositeItemState`
  - `controls::state::MenuPath`
- Numeric/range primitives:
  - `controls::value::ControlRange`
- Template system:
  - `controls::template::ControlTemplate<T, M>`
  - `controls::template::TemplateWithModifiers<M>`
  - many modules define `*Template` + `*TemplateHandlers`
- Theming:
  - semantic token tree (`LumaTheme` / `ThemeTokens`) with palette/metrics/typography/elevation
  - control-specific theme traits resolve strongly typed appearance structs

## Error Handling Conventions

- Fallible setup/parsing APIs return `anyhow::Result`.
  - examples: `init::init`, theme parsing/conversion pipeline in `theme/tokens.rs`
- Error contexts are added in parsing logic (`context`, `with_context`).
- Application startup handles errors by printing diagnostics in `apps/gallery/src/main.rs`.
- There is one deliberate `expect(...)` in theme loading path:
  - `LumaTheme::native()` expects embedded default theme TOML to parse.

## Async / Runtime Assumptions

- Runtime model is GPUI entity/event driven (subscriptions + `cx.notify()`).
- Some controls spawn async tasks via `cx.spawn(...)` (notably `textfield` and `textarea` for caret blink/selection scroll behavior).
- No Tokio runtime assumptions are encoded in this repo; behavior depends on GPUI runtime.
- Keyboard behavior depends on key context wiring and bound actions.

## Test Strategy (Current State)

- Unit tests are concentrated in SDK modules, especially behavior/state/math/template placement.
- Approximate test count in Rust sources: 85 `#[test]` functions.
- Strongly tested areas include:
  - text editing behavior (`controls/text/editing.rs`)
  - textarea state/control logic
  - control group selection/normalization
  - menu navigation
  - selector/popup template placement
  - range math (`ControlRange`)
  - keybinding profile stability
  - theme registry/token integrity checks
- Gallery app currently has no dedicated tests; it is primarily an interactive/manual validation surface.

## Known Risks

- `LumaTheme::native()` can panic if embedded TOML becomes invalid.
- Theme usage registry is manually curated (`theme/radix/usage.rs`), so omissions are possible when adding new controls/themes.
- Autocomplete and combobox behavior are similar but separate, creating drift risk.
- Large stateful modules (`textarea`, `textfield`, `navigation_sidebar`) are regression-prone without broader integration tests.
- Gallery routing uses string IDs; accidental mismatches are possible.
- Gallery lacks automated tests, so regressions are mainly caught manually.

## Quick Navigation

- SDK entry: `crates/sdk/src/lib.rs`
- Controls root: `crates/sdk/src/controls/mod.rs`
- Theme root: `crates/sdk/src/theme/mod.rs`
- Gallery entry: `apps/gallery/src/main.rs`
- Gallery routing/registry: `apps/gallery/src/gallery/panes/registry.rs`

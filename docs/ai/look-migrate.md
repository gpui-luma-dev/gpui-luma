# Look Migration Plan: Transitioning to Shadcn Style System

This document outlines the migration strategy to replace the legacy styling configurations with the type-safe, compile-time `ShadcnLook` across GPUI-Luma applications and SDK components.

---

## 1. Comparing View Implementations

Below is a side-by-side comparison of migrating a list of cards from the previous layout style to the new context-bound `ShadcnLook` approach:

### BEFORE (Endless boilerplate, magic values, no alignment)
```rust
fn member_row(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
    chrome: gpui_luma::theme::LumaChrome,
    avatar_bg: gpui::Hsla,
) -> impl IntoElement {
    hstack! {
        gap=10 align=center;
        avatar_circle(initials, 32.0, avatar_bg, chrome.title_text),
        vstack! {
            gap=2;
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .child(name),
            div()
                .text_size(px(11.0))
                .line_height(px(14.0))
                .text_color(chrome.muted_text)
                .child(email),
        }
    }
}
```

### AFTER (Direct mapping, zero magic strings, extremely readable)
```rust
fn member_row(
    initials: &'static str,
    name: &'static str,
    email: &'static str,
) -> impl IntoElement {
    hstack! {
        gap_cn=2.5 align=center;                       // Spacing grid (2.5 * --spacing)
        avatar_circle(initials, 32.0),
        vstack! {
            gap_cn=0.5;                                // Spacing grid (0.5 * --spacing)
            div()
                .text_size_cn(ShadcnTextSize::Sm)      // Typographic Scale
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_cn(ShadcnToken::Foreground)
                .font_cn(ShadcnFont::Sans)
                .child(name),
            div()
                .text_size_cn(ShadcnTextSize::Xs)      // Typographic Scale
                .line_height(px(14.0))
                .text_cn(ShadcnToken::MutedForeground)
                .font_cn(ShadcnFont::Sans)
                .child(email),
        }
    }
}
```

---

## 2. Crate Isolation Migration Strategy

To maintain a strict separation of concerns and avoid namespace pollution in the core SDK, we adopt a Crate Isolation Strategy. Both appearance systems are defined as downstream crates depending on the core SDK:

                           ┌─────────────────────────┐
                           │   crates/sdk (Core)     │
                           └───────────┬─────────────┘
                                       │
                      ┌────────────────┴────────────────┐
                      ▼                                 ▼
         ┌─────────────────────────┐       ┌─────────────────────────┐
         │   crates/theme-radix    │       │   crates/look-shadcn    │
         │  (gpui-luma-theme-radix)│       │ (gpui-luma-look-shadcn) │
         └─────────────────────────┘       └─────────────────────────┘

### 1. Crate Partitioning [COMPLETED]
The new downstream look crate `crates/look-shadcn` has been successfully created with a library structure and declared in `Cargo.toml`. 

All 31 control appearance templates, shadow resolvers, and base styling extension methods have been ported from the legacy radix code and successfully compiled and tested under the `gpui-luma-look-shadcn` package.

### 2. Phase A: Migrate Theme Studio Panels
Migrate the isolated view pages under `apps/theme-studio/src/studio/panels/` (`team.rs`, `chat.rs`, etc.) and the application shell to consume `ShadcnLook` and its builder extension APIs.

#### Step 1: Update Cargo Dependencies
Add `gpui-luma-look-shadcn` to the dependencies of the downstream apps:
* In `apps/theme-studio/Cargo.toml`:
  ```toml
  gpui-luma-look-shadcn = { path = "../../crates/look-shadcn" }
  ```
* In `apps/gallery/Cargo.toml`:
  ```toml
  gpui-luma-look-shadcn = { path = "../../crates/look-shadcn" }
  ```

#### Step 2: Implement theme loading in `theme.rs`
Update [apps/theme-studio/src/theme.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/theme.rs) to resolve and return `Arc<ShadcnLook>`:
```rust
use gpui_luma_look_shadcn::ShadcnLook;

impl StudioThemeChoice {
    pub fn shadcn_look(self) -> Arc<ShadcnLook> {
        match self {
            Self::Default => Arc::new(ShadcnLook::native()),
            Self::Named(stem) => {
                let path = theme_css_path(&stem);
                Arc::new(
                    ShadcnLook::from_css_path(&path)
                        .unwrap_or_else(|err| panic!("parse shadcn theme {}: {err}", path.display())),
                )
            }
        }
    }
}
```

#### Step 3: Transition `ThemeStudioApp` state to `ShadcnLook`
Update [apps/theme-studio/src/studio/app.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/app.rs) to use `ShadcnLook` as the primary layout styling resolver:
* Replace the field `pub(super) radix_theme: Arc<RadixTheme>` with `pub(super) look: Arc<ShadcnLook>`.
* Replace `StudioThemeChoice::radix_theme()` with `.shadcn_look()` during initialization.
* Update `DemoControls::spawn`, `ThemeSidebar::new`, and `ResizablePanels` themes to consume the template providers of `self.look`.

#### Step 4: Migrate Individual Panels
For each layout panel in `apps/theme-studio/src/studio/panels/` (e.g., [team.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/team.rs)):
* Change imports from `use gpui_luma_theme_radix::prelude::*;` to `use gpui_luma_look_shadcn::prelude::*;`.
* Change argument signatures from `radix_theme: Arc<RadixTheme>` to `look: Arc<ShadcnLook>`.
* Replace all explicit theme style modifiers (like `.primary(&self.radix_theme)`) with `.primary(&self.look)`.
* Wrap the top-level rendering calls inside `with_look(&self.look, || { ... })` to allow thread-local background/foreground and spacing calculations (`bg_cn`, `gap_cn`, etc.) to resolve dynamically.

> [!NOTE]
> **Migration Quirk (Radix Trait Leftovers)**:
> Due to the copy-paste scaffolding of `crates/look-shadcn/src/controls/ext.rs` from `theme-radix`, the styling extension traits (`ShadcnButtonStyleExt`, `ShadcnCheckboxStyleExt`, etc.) still expose the method name `.radix_style(...)` instead of `.shadcn_style(...)` or `.look_style(...)`.
> 
> * **Standard styling** (e.g. `.primary(&look)`) is unaffected.
> * **Parameterized styling** (e.g. custom button style variations) must use `.radix_style(&look, ShadcnButtonStyle::Outline)` for now:
>   ```rust
>   // Example custom button setup
>   look.button("cancel").radix_style(&look, ShadcnButtonStyle::Outline)
>   ```

### 3. Phase B: Dynamic Resolution Mapping
Implement state/interaction math and fallback chains within the `ShadcnLook` resolver layer rather than the views. Components query state layers via:
```rust
look.resolve_color_state(ShadcnToken::Primary, InteractionLayer::Hovered)
```
This preserves theme-resilience for missing custom properties in raw CSS imports while keeping client view code clean.

### 4. Phase C: SDK Migration and Deprecation
Refactor the remaining core controls inside `crates/sdk/src/controls/` to consume `ShadcnLook` and compile-time tokens. Once all control templates and the gallery app compile on the new model, we can safely deprecate and remove the legacy `gpui-luma-theme-radix` crate from the workspace.

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

### 1. Crate Partitioning
Create the parallel look crate `crates/look-shadcn` with a library structure and declare it in `Cargo.toml`:

```toml
[package]
name = "gpui-luma-look-shadcn"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = { workspace = true }
gpui-luma = { path = "../sdk" }
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
toml = { workspace = true }
lucide-icons = { workspace = true }
```

Add `"crates/look-shadcn"` to the workspace members in the root `Cargo.toml`.

### 2. Phase A: Migrate Theme Studio Panels
Migrate the isolated view pages under `apps/theme-studio/src/studio/panels/` (`team.rs`, `chat.rs`, etc.) to consume `ShadcnLook` and the type-safe style extensions. 
* This allows visual validation against the custom tweakcn `.css` exports (like `claude.css` and `jarvis.css`) within the customization workspace.
* Wrap the top-level rendering calls inside `with_look(look, || { ... })`.

### 3. Phase B: Dynamic Resolution Mapping
Implement state/interaction math and fallback chains within the `ShadcnLook` resolver layer rather than the views. Components query state layers via:
```rust
look.resolve_color_state(ShadcnToken::Primary, InteractionLayer::Hovered)
```
This preserves theme-resilience for missing custom properties in raw CSS imports while keeping client view code clean.

### 4. Phase C: SDK Migration and Deprecation
Refactor the remaining core controls inside `crates/sdk/src/controls/` to consume `ShadcnLook` and compile-time tokens. Once all control templates and the gallery app compile on the new model, we can safely deprecate and remove the legacy `gpui-luma-theme-radix` crate from the workspace.

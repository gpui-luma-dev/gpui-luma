# Look-Specific Extensions & Add-On Components

To maintain a clean, look-agnostic SDK, components and styling variations that are specific to a particular design language (like Shadcn) should not clutter the SDK. Instead, they are implemented as **look-specific extensions** inside the look crate (`crates/look-shadcn`).

This document describes the use cases, code patterns, and integration guidelines for these look-only add-ons (such as specialized buttons, badges, and layout containers).

---

## 1. Architectural Concept: Look-Specific Extension Trait
The SDK defines core interactive templates. The look crate extends the SDK's theme factory by implementing the `ShadcnLookControlExt` trait:

```rust
// crates/look-shadcn/src/controls/ext.rs
pub trait ShadcnLookControlExt {
    // 1. Core SDK controls (pre-bound to Shadcn templates)
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;

    // 2. Specialized look-specific button variations
    fn destructive_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn warning_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;

    // 3. Look-only compound widgets
    fn card(&self, title: impl Into<String>, subtitle: impl Into<String>, content: Div) -> Card;
    fn badge(&self, label: impl Into<String>, variant: BadgeVariant) -> Badge;
}
```

This pattern keeps the SDK **standalone and clean** while giving downstream applications a unified, theme-driven way to spawn both core and specialized widgets.

---

## 2. Use Case 1: Specialized Button/Toggle Variations
* **Examples**: `destructive_button()`, `warning_button()`, `success_button()`.
* **The Problem**: The SDK button palette (`ButtonFamilyPalette`) is look-agnostic, and its style enum only covers basic variants (`Primary`, `Secondary`, `Outline`, `Ghost`).
* **The Extension Solution**:
  * We add the look-specific variants (`Destructive`, `Warning`) to the style enum in `crates/look-shadcn/src/controls/button.rs`.
  * We author corresponding color rules in `style.toml`.
  * The look resolver resolves these custom variants into concrete background and foreground colors.
  * The SDK template renders them natively without ever knowing that a "destructive" button style exists.

```toml
# style.toml - Mapped entirely in the look crate
[[button.color_rules]]
style = "destructive"
layer = "default"
background = "destructive"
foreground = "destructive-foreground"
```

---

## 3. Use Case 2: Compound Look-Only Containers (e.g., `Card`)
* **Examples**: `Card { title, subtitle, content }`.
* **The Problem**: A `Card` is a purely visual block. Moving it to the SDK would force us to write abstract templates for a simple static layout.
* **The Extension Solution**:
  * We define the GPUI layout structure and implement `IntoElement` for `Card` entirely inside `look-shadcn/src/controls/card.rs`.
  * The constructor takes a pre-resolved `CardPalette` containing colors and metrics queried from the stylesheet:
  
```rust
// crates/look-shadcn/src/controls/card.rs
pub struct Card {
    title: String,
    subtitle: String,
    content: Div,
    palette: CardPalette,
}

impl IntoElement for Card {
    type Element = Div;

    fn into_element(self, cx: &mut WindowContext) -> Self::Element {
        div()
            .bg(self.palette.background)
            .border_1()
            .border_color(self.palette.border.unwrap_or(gpui::transparent()))
            .rounded(px(self.palette.radius))
            .child( /* Render title, subtitle, content */ )
    }
}
```

---

## 4. Use Case 3: Toggle Variations & Badges
* **Examples**: `Badge` (pill indicator with `default`, `secondary`, `outline`, `destructive` styles).
* **The Problem**: A Badge is structurally a non-interactive pill, or sometimes a tiny toggled indicator. It has no behavior, only visual skin.
* **The Extension Solution**:
  * Implement the Badge element directly in `look-shadcn/src/controls/badge.rs`.
  * Use the extension trait to build it from the active look, retrieving colors (e.g. `bg_color`, `text_color`) and rounded metrics directly from `style.toml`:

```toml
# style.toml - Badge Stylesheet Section
[[badge.color_rules]]
style = "default"
background = "primary"
foreground = "primary-foreground"

[[badge.color_rules]]
style = "destructive"
background = "destructive"
foreground = "destructive-foreground"
```

---

## 5. Summary of Benefits
1. **SDK Standalone Integrity**: SDK compiles without carrying look-specific visual variations or metadata dependencies.
2. **Simplified App Code**: App shells simply call `look.card(...)` or `look.badge(...)` in their rendering trees.
3. **Flexible Theme Customization**: Design rules and color overrides for all specialized widgets remain centralized in `style.toml`.

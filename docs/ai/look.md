# Type-Safe Shadcn Theme Mappings in GPUI

> [!IMPORTANT]
> **Core Architectural Philosophy**
> We are not proving that “the SDK supports many themes.” We are proving that **“the SDK supports multiple appearance systems, each with their own theme sources and specializations.”**
> 
> The `shadcn` styling system introduces a **Look** (the runtime appearance system) called `ShadcnLook` that parses theme data files (such as `CssTokenCatalog`) and applies a Tailwind/shadcn utility-based layout system to elements. The legacy `RadixTheme` / `gpui-luma-theme-radix` crate has been removed; `ShadcnLook` is the sole downstream look crate.
> 
> **Important Constraint**: `ShadcnLook` lives in the downstream crate `gpui-luma-look-shadcn` to keep the core SDK (`crates/sdk`) completely look-agnostic.

---

## 1. Terminology Schema

To keep the architecture clear, we distinguish between the raw data configuration (the Theme) and the new runtime rendering resolver (the Look):

| Term | Target Concept | Role in Codebase |
| :--- | :--- | :--- |
| **Theme / Tokens** | `CssTokenCatalog`, `ThemeTokens` | **The Static Data**: Raw key-value values parsed from CSS (e.g. `claude.css`) or TOML configurations. |
| **Look** | `ShadcnLook` | **The Appearance System**: The new runtime engine that takes theme tokens and resolves them into layout heights, font sizes, colors, and shadows. |
| **Look Controls** | `ShadcnLookControlExt` | **The Control Spawn APIs**: Extension traits that bind SDK elements to templates defined by the Shadcn Look. |

---

## 2. Design Token Enums

To enforce compile-time safety and eliminate all magic string literals, design parameters are modeled as strongly-typed Rust enums:

### Colors & Opacity

```rust
use gpui::{Hsla, Styled};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnToken {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
}

#[derive(Clone, Copy, Debug)]
pub struct ShadcnStyle {
    pub token: ShadcnToken,
    pub opacity: Option<f32>,
}

impl ShadcnToken {
    /// Chainable opacity modifier matching the `/opacity` Tailwind suffix
    /// e.g. `ShadcnToken::Input.opacity(0.3)` -> `bg-input/30`
    pub fn opacity(self, alpha: f32) -> ShadcnStyle {
        ShadcnStyle {
            token: self,
            opacity: Some(alpha),
        }
    }
}

impl From<ShadcnToken> for ShadcnStyle {
    fn from(token: ShadcnToken) -> Self {
        ShadcnStyle {
            token,
            opacity: None,
        }
    }
}
```

### Metrics & Typography

```rust
/// Font families defined in the CSS layout (e.g. `--font-sans`, `--font-mono`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnFont {
    Sans,
    Serif,
    Mono,
}

/// Border radius slots computed relative to the base theme `--radius`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnRadius {
    None,
    Sm, // calc(var(--radius) - 4px)
    Md, // calc(var(--radius) - 2px)
    Lg, // var(--radius)
    Xl, // calc(var(--radius) + 4px)
}

/// Box shadow elevations defined in the CSS (e.g. `--shadow-sm`, `--shadow-md`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnShadow {
    None,
    TwoXs,
    Xs,
    Sm,
    Default,
    Md,
    Lg,
    Xl,
    TwoXl,
}

/// Standardized typographic scale matching Tailwind text sizes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadcnTextSize {
    Xs,   // 11.0px (e.g., helpers, captions)
    Sm,   // 12.5px (standard labels, descriptions)
    Base, // 14.0px (body text)
    Lg,   // 16.0px
    Xl,   // 18.0px
    TwoXl,// 20.0px (titles)
}
```

---

## 3. Resolving in the Look Layer

The `ShadcnLook` resolves parsed metrics, typography, and color variables into GPUI native values.

```rust
impl ShadcnLook {
    /// Resolves colors directly from the loaded palette
    pub fn color(&self, token: ShadcnToken) -> Hsla {
        let palette = &self.mode_tokens().palette;
        match token {
            ShadcnToken::Background => palette.app_background,
            ShadcnToken::Foreground => palette.app_foreground,
            ShadcnToken::Card => palette.panel_background,
            ShadcnToken::CardForeground => palette.body_text,
            ShadcnToken::Popover => palette.panel_background,
            ShadcnToken::PopoverForeground => palette.body_text,
            ShadcnToken::Primary => palette.primary.background,
            ShadcnToken::PrimaryForeground => palette.primary.foreground,
            ShadcnToken::Secondary => palette.secondary.background,
            ShadcnToken::SecondaryForeground => palette.secondary.foreground,
            ShadcnToken::Muted => palette.muted_background,
            ShadcnToken::MutedForeground => palette.app_muted_foreground,
            ShadcnToken::Accent => palette.ghost.hover_background,
            ShadcnToken::AccentForeground => palette.ghost.foreground,
            ShadcnToken::Destructive => palette.disabled_background,
            ShadcnToken::DestructiveForeground => palette.disabled_foreground,
            ShadcnToken::Border => palette.border_default,
            ShadcnToken::Input => palette.input_background,
            ShadcnToken::Ring => palette.focus_ring,
        }
    }

    /// Resolves fonts to their loaded families
    pub fn font(&self, role: ShadcnFont) -> &str {
        match role {
            ShadcnFont::Sans => self.token("font-sans").unwrap_or("sans-serif"),
            ShadcnFont::Serif => self.token("font-serif").unwrap_or("serif"),
            ShadcnFont::Mono => self.token("font-mono").unwrap_or("monospace"),
        }
    }

    /// Resolves border-radius to raw pixels (f32) based on the current theme radius
    pub fn radius(&self, role: ShadcnRadius) -> f32 {
        let base_radius = self.parse_pixel_token("radius").unwrap_or(8.0);
        match role {
            ShadcnRadius::None => 0.0,
            ShadcnRadius::Sm => (base_radius - 4.0).max(0.0),
            ShadcnRadius::Md => (base_radius - 2.0).max(0.0),
            ShadcnRadius::Lg => base_radius,
            ShadcnRadius::Xl => base_radius + 4.0,
        }
    }

    /// Resolves custom shadow structures for GPUI box-shadow arrays
    pub fn shadow(&self, role: ShadcnShadow) -> Vec<gpui::Shadow> {
        let shadow_key = match role {
            ShadcnShadow::None => return vec![],
            ShadcnShadow::TwoXs => "shadow-2xs",
            ShadcnShadow::Xs => "shadow-xs",
            ShadcnShadow::Sm => "shadow-sm",
            ShadcnShadow::Default => "shadow",
            ShadcnShadow::Md => "shadow-md",
            ShadcnShadow::Lg => "shadow-lg",
            ShadcnShadow::Xl => "shadow-xl",
            ShadcnShadow::TwoXl => "shadow-2xl",
        };
        
        self.parse_shadow_token(shadow_key).unwrap_or_default()
    }
}
```

---

## 4. Ergonomics: Scoped Thread-Local Look Pattern

To avoid repeating the Look variable in every style builder call, the active Look context is stored in thread-local storage during the rendering phase. A layout block is wrapped inside `with_look(look, || { ... })`, and the extension methods automatically resolve styling parameters without carrying arguments.

```rust
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    static ACTIVE_LOOK: RefCell<Option<Arc<ShadcnLook>>> = RefCell::new(None);
}

/// Binds the active Look to the thread's scope during layout/render block execution
pub fn with_look<R>(look: &Arc<ShadcnLook>, f: impl FnOnce() -> R) -> R {
    ACTIVE_LOOK.with(|cell| {
        *cell.borrow_mut() = Some(look.clone());
    });
    let result = f();
    ACTIVE_LOOK.with(|cell| {
        *cell.borrow_mut() = None;
    });
    result
}

// Styling extensions retrieve the active Look implicitly from thread local storage
pub trait ShadcnElementExt: Styled + Sized {
    // --- Base Color Directives ---
    fn bg_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let mut color = look.color(style.token);
                if let Some(alpha) = style.opacity {
                    color.a = alpha;
                }
                self.bg(color)
            } else {
                self
            }
        })
    }

    fn text_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let mut color = look.color(style.token);
                if let Some(alpha) = style.opacity {
                    color.a = alpha;
                }
                self.text_color(color)
            } else {
                self
            }
        })
    }

    fn border_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        let style = style.into();
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let mut color = look.color(style.token);
                if let Some(alpha) = style.opacity {
                    color.a = alpha;
                }
                self.border_color(color)
            } else {
                self
            }
        })
    }

    // --- State Modifiers (Tailwind hover/active/focus equivalents) ---
    fn hover_bg_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        self.hover(move |s| s.bg_cn(style))
    }

    fn hover_text_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        self.hover(move |s| s.text_cn(style))
    }

    fn active_bg_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        self.active(move |s| s.bg_cn(style))
    }

    fn focus_border_cn(self, style: impl Into<ShadcnStyle>) -> Self {
        self.focus(move |s| s.border_cn(style))
    }

    /// Draws focus ring using active ring and background colors (matches ring-ring focus styles)
    fn focus_ring_cn(self) -> Self {
        self.focus(|s| {
            ACTIVE_LOOK.with(|cell| {
                if let Some(ref look) = *cell.borrow() {
                    s.outline_color(look.color(ShadcnToken::Ring))
                } else {
                    s
                }
            })
        })
    }

    // --- Typography & Metrics Directives ---
    fn rounded_cn(self, role: ShadcnRadius) -> Self {
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                self.rounded(px(look.radius(role)))
            } else {
                self
            }
        })
    }

    fn font_cn(self, role: ShadcnFont) -> Self {
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                self.font_family(look.font(role))
            } else {
                self
            }
        })
    }

    fn text_size_cn(self, size: ShadcnTextSize) -> Self {
        let px_size = match size {
            ShadcnTextSize::Xs => 11.0,
            ShadcnTextSize::Sm => 12.5,
            ShadcnTextSize::Base => 14.0,
            ShadcnTextSize::Lg => 16.0,
            ShadcnTextSize::Xl => 18.0,
            ShadcnTextSize::TwoXl => 20.0,
        };
        self.text_size(px(px_size))
    }

    // --- Spacing Scale Helpers ---
    fn gap_cn(self, step: f32) -> Self {
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.gap(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn p_cn(self, step: f32) -> Self {
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.p(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn px_cn(self, step: f32) -> Self {
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.px(px(step * spacing))
            } else {
                self
            }
        })
    }

    fn py_cn(self, step: f32) -> Self {
        ACTIVE_LOOK.with(|cell| {
            if let Some(ref look) = *cell.borrow() {
                let spacing = look.parse_pixel_token("spacing").unwrap_or(4.0);
                self.py(px(step * spacing))
            } else {
                self
            }
        })
    }
}

impl<S: Styled> ShadcnElementExt for S {}
```

---

## 5. Visual Parity: The Avatar Card Example

### The Component Interface

By leveraging thread-local context, the visual mapping matches the simplicity of Tailwind/shadcn:

```rust
pub fn avatar_circle(initials: &'static str, size: f32) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::SEMIBOLD)
        .bg_cn(ShadcnToken::Muted)                  // Implicit look resolution
        .text_cn(ShadcnToken::MutedForeground)       // Implicit look resolution
        .font_cn(ShadcnFont::Mono)                  // Auto-resolves JetBrains Mono
        .rounded_cn(ShadcnRadius::Lg)
        .child(initials)
}
```


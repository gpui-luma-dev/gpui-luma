# Issue #1: Typography Standards and Sizing Scales

## Description
Throughout the workspace, there are numerous occurrences of hardcoded font-sizes (e.g., `.text_size(px(14.0))`, `.text_size(px(11.0))`) in the application shells (`apps/gallery` and `apps/theme-studio`) and in several SDK controls. 

This ad-hoc styling bypasses the centralized design token system (`LumaTypography`) and stylesheet mappings (`style.toml`). As a result, switching themes (such as from *Modern Minimal* to *Retro Arcade* or *Jarvis*) or adjusting scaling factors does not consistently scale or visual-style text elements. 

To solve this, we need to introduce a unified, semantic typography system. This system will define HTML-like headings and a standardized Tailwind-aligned size scale (`xs`...`2xl`).

### Code Audit: Hardcoded Typography Examples
Below are representative examples of hardcoded typography found in the workspace:

#### 1. In `apps/gallery`
* **Pane Titles & Introduction**:
  * [introduction/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/introduction/pane.rs): Large title hardcoded at `.text_size(px(44.0))` and description heading at `.text_size(px(17.0))`.
* **Component Panes**:
  * [selection_panel/layout.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/selection_panel/layout.rs): Uses `.text_size(px(15.0))` and `.text_size(px(12.0))` for headers.
  * [badge/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/badge/pane.rs): Uses `.text_size(px(12.0))` for badge labels.
  * [palette/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/palette/pane.rs): Section titles use `.text_size(px(20.0))` and descriptions use `.text_size(px(13.0))`.
  * [dock_panel/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/dock_panel/pane.rs): Uses `.text_size(px(20.0))` and `.text_size(px(13.0))`.

#### 2. In `apps/theme-studio`
* **App Shell Headers**:
  * [studio/app.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/app.rs): The main header "Luma Theme Studio" is hardcoded to `.text_size(px(14.0))` with weight `SEMIBOLD`, and description metadata uses `.text_size(px(11.0))`.
* **Panes & Previews**:
  * [studio/panels/dashboard.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/dashboard.rs): Dashboard headers are hardcoded to `.text_size(px(16.0))` and `.text_size(px(14.0))`.
  * [studio/panels/team.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/team.rs): Uses `.text_size(px(16.0))` and `.text_size(px(12.0))` for member cards.
  * [studio/panels/palette.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/palette.rs): Hardcoded titles at `.text_size(px(20.0))` and secondary metrics labels at `.text_size(px(12.0))`.

#### 3. In `crates/sdk`
* **Controls & Helpers**:
  * [controls/label.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/label.rs): The standard `field_label` is hardcoded to `.text_size(px(11.0))` and `.line_height(px(14.0))`.
  * [controls/navigation_sidebar/template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/navigation_sidebar/template.rs): Sidebar headers use hardcoded constants `const TITLE_FONT_SIZE: f32 = 14.0;` and `const SUBTITLE_FONT_SIZE: f32 = 12.0;`.

---

## Proposed Solution

To resolve this, we will introduce a standard typography module and associated layout scaling extensions.

### 1. Sizing Scales (HTML Headings & Tailwind Scale)
We will introduce two semantic scales to cover different use cases:

#### A. HTML-Style Headings
For structural layouts, dashboard panels, and static copy:
* **`h1`**: Primary Page/Hero Title (e.g., `32px` to `44px`, `Bold` / `ExtraBold`)
* **`h2`**: Main Section Header (e.g., `24px` to `28px`, `SemiBold`)
* **`h3`**: Sub-section / Panel Header (e.g., `18px` to `20px`, `SemiBold`)
* **`h4`**: Component Header (e.g., `15px` to `16px`, `Medium` / `SemiBold`)
* **`p`**: Standard Body Copy (e.g., `13px` to `14px`, `Normal`)

#### B. Sizing Scale (Tailwind-Aligned: `xs` to `2xl`)
A typographic scale aligned with standard Tailwind size increments:
* **`xs`**: Extra Small (`11px` / `0.75rem`) -> Captions, fine print, indicators.
* **`sm`**: Small (`12.5px` / `0.875rem`) -> Helper labels, secondary descriptions, badge items.
* **`base` / `md`**: Medium / Default (`14px` / `1rem`) -> Default body copy, form inputs, buttons.
* **`lg`**: Large (`16px` / `1.125rem`) -> List titles, primary control fields.
* **`xl`**: Extra Large (`18px` / `1.25rem`) -> Dialog headers, card titles.
* **`2xl`**: 2x Large (`20px` / `1.5rem`) -> Main subheadings, page subsection titles.

### 2. Design Invariant: Theme-Aware Sizing Resolution
To guarantee that typography responds dynamically to user changes (such as theme switching, dark mode, or scaling modifications):
* **No hardcoded sizes**: The helper methods and extensions **must not** define or return hardcoded float or pixel constants.
* **Context Resolution**: Methods must query the active `LumaTheme` context (for SDK fallback) or `ShadcnLook` catalog (for themed look-shadcn components) to determine the logical bounds and weights for the specified scale step.

### 3. API Enhancements & Integration
To make these easy to use, we will introduce:

#### A. Typography Extension Trait (`LumaTypographyExt`)
Add extension methods to GPUI's `Styled` trait (implemented inside `look-shadcn` for themed components):
```rust
pub trait LumaTypographyExt: Styled + Sized {
    // Semantic HTML-like headings
    fn text_h1(self) -> Self;
    fn text_h2(self) -> Self;
    fn text_h3(self) -> Self;
    fn text_h4(self) -> Self;
    fn text_p(self) -> Self;

    // Tailwind-aligned size scale (T-Scale)
    fn text_xs(self) -> Self;
    fn text_sm(self) -> Self;
    fn text_md(self) -> Self; // or text_base
    fn text_lg(self) -> Self;
    fn text_xl(self) -> Self;
    fn text_2xl(self) -> Self;
}
```

#### B. Inline Functional Helpers (in `look-shadcn`)
For simple text rendering:
```rust
pub fn h1(text: impl Into<SharedString>) -> impl IntoElement;
pub fn h2(text: impl Into<SharedString>) -> impl IntoElement;
pub fn h3(text: impl Into<SharedString>) -> impl IntoElement;
pub fn p(text: impl Into<SharedString>) -> impl IntoElement;
```

#### C. Fluent Styling Styler Closures (in `look-shadcn`)
To enable fluent inline styling transitions, dynamic overrides, and re-usable style classes without breaking elements' builder chains, we will introduce styler closure helpers:
* **`.styler(closure)`**: Modifies element styling inline.
* **`.style_with(closure)`**: Modifies element styling dynamically using look-shadcn context variables.

##### Trait Definition
```rust
pub trait LumaStylerExt: Styled + Sized {
    fn styler<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self;
}

pub trait ShadcnStylerExt: Styled + Sized {
    fn style_with<F>(self, f: F) -> Self
    where
        F: FnOnce(Self, &ShadcnLook) -> Self;
}
```

##### Usage Examples
```rust
// 1. Simple inline conditional styling
div().styler(|e| if is_active { e.text_sm().font_bold() } else { e.text_xs() })

// 2. Reusing a style preset function
fn card_header(e: div) -> div { e.text_lg().font_semibold() }
div().styler(card_header).child("Account Settings")

// 3. Dynamic lookup with context variables
div().style_with(|e, look| e.text_size(px(look.text_size(ShadcnTextSize::Base))))
```

---

## Tasks

### Phase 1: SDK Core Changes (`crates/sdk`)
- [ ] Ensure core text styles (`TextTokens` in [tokens.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/theme/tokens.rs)) support a baseline scale for standard sizing models.
- [ ] Define helper structures or baseline traits if needed for SDK elements to scale size parameters dynamically.

### Phase 2: Downstream Look-Shadcn Sizing & Extensions (`crates/look-shadcn`)
- [ ] Register new text-size rule mappings (supporting custom CSS variable extraction for text size keys) in the stylesheet resolver.
- [ ] Implement `LumaTypographyExt` and layout helper styles on `ShadcnElementExt` (in [ext.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/ext.rs)) to resolve size scales against active CSS variables.
- [ ] Implement `LumaStylerExt` and `ShadcnStylerExt` on element extension helpers to support inline and context closures.
- [ ] Add the functional HTML-style element helpers (`h1`, `h2`, `h3`, `p`) in `look-shadcn`.
- [ ] Map standard typography configurations in `crates/look-shadcn/assets/style.toml`.

### Phase 3: Codebase Refactor & Cleanup
- [ ] Refactor gallery panes (especially `introduction`, `selection_panel`, `palette`, and pane headers) to replace hardcoded `.text_size(px(...))` with the new typography extension utilities (using `.styler` or `.style_with` closures for clean layout rendering where applicable).
- [ ] Refactor `theme-studio` panels (like `dashboard`, `team`, `palette`) to use standard headings (`h1`..`h4`) and spacing methods (leveraging `.styler` closures to avoid local mutable element bindings).
- [ ] Update SDK internal controls (like `field_label` in `controls/label.rs` and `navigation_sidebar/template.rs`) to use the new tokens.

---

## Acceptance Criteria
- No hardcoded `.text_size(px(14.0))` or similar manual sizes are used for standard labels, titles, and paragraphs in gallery previews and workspace dashboard views.
- Sizing changes dynamically when switching between themes or scale factors.
- The project builds cleanly with `cargo check` and runs without regression in text bounds or baseline alignment.

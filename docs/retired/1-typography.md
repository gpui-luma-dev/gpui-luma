# Issue #1: Typography Standards and Sizing Scales

## Description
Throughout the workspace, there are numerous occurrences of hardcoded font-sizes (e.g., `.text_size(px(14.0))`, `.text_size(px(11.0))`) in the application shells (`apps/gallery` and `apps/luma-studio`) and in several SDK controls.

This ad-hoc styling bypasses the centralized design token system (`LumaTypography`) and stylesheet mappings (`style.toml`). As a result, switching themes (such as from *Modern Minimal* to *Retro Arcade* or *Jarvis*) or adjusting scaling factors does not consistently scale or visual-style text elements.

To solve this, we need to introduce a unified, semantic typography system. This system should define HTML-like headings and a standardized size scale, while also being explicit about **where GPUI's built-in text helpers are allowed** and **where SDK-owned typography must be used instead**.

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

#### 2. In `apps/luma-studio`
* **App Shell Headers**:
  * [studio/app.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/app.rs): The main header "Luma Studio" is hardcoded to `.text_size(px(14.0))` with weight `SEMIBOLD`, and description metadata uses `.text_size(px(11.0))`.
* **Panes & Previews**:
  * [studio/panels/dashboard.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panels/dashboard.rs): Dashboard headers are hardcoded to `.text_size(px(16.0))` and `.text_size(px(14.0))`.
  * [studio/panels/team.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panels/team.rs): Uses `.text_size(px(16.0))` and `.text_size(px(12.0))` for member cards.
  * [studio/panels/palette.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/panels/palette.rs): Hardcoded titles at `.text_size(px(20.0))` and secondary metrics labels at `.text_size(px(12.0))`.

#### 3. In `crates/sdk`
* **Controls & Helpers**:
  * [controls/label.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/label.rs): The standard `field_label` is hardcoded to `.text_size(px(11.0))` and `.line_height(px(14.0))`.
  * [controls/navigation_sidebar/template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/navigation_sidebar/template.rs): Sidebar headers use hardcoded constants `const TITLE_FONT_SIZE: f32 = 14.0;` and `const SUBTITLE_FONT_SIZE: f32 = 12.0;`.

---

## Proposed Solution

To resolve this, we will introduce a standard typography module and associated layout scaling extensions, **with a strict split between app usage and SDK usage**.

## 1. Two Typography Sources, Two Rules

### A. App Code (`apps/*`)
App shells may use **GPUI's built-in fixed text scale helpers** when that is sufficient and desirable for grepability and debugging:

- `text_xs()`
- `text_sm()`
- `text_base()`
- `text_lg()`
- `text_xl()`
- `text_2xl()`

This keeps app code pragmatic and makes it easy to audit later where raw GPUI sizing behavior is in use.

**Important:** these GPUI helpers are fixed Tailwind-like sizes and are **not** the SDK's theme-aware typography API.

### B. SDK / Look Layer (`crates/sdk`, `crates/look-shadcn`)
SDK and look-layer code should **not** use GPUI's fixed `text_sm()` / `text_lg()` helpers for normal semantic UI text.

Instead, SDK/look code must resolve typography through:

- `LumaTypography`
- `LumaTextStyle`
- `ShadcnLook::typography_role(...)`
- `ShadcnLook::typography_scale(...)`
- SDK-owned helper names that do **not** collide with GPUI methods

This keeps typography policy centralized in the SDK and prevents product controls from drifting into mixed, partially manual text styling.

---

## 2. Sizing Scales (HTML Headings & Shared Scale)
We will introduce two semantic layers to cover different use cases.

### A. HTML-Style Headings
For structural layouts, dashboard panels, and static copy:

* **`h1`**: Primary page/hero title (e.g. `32px` to `44px`, `Bold` / `ExtraBold`)
* **`h2`**: Main section header (e.g. `24px` to `28px`, `SemiBold`)
* **`h3`**: Sub-section / panel header (e.g. `18px` to `20px`, `SemiBold`)
* **`h4`**: Component header (e.g. `15px` to `16px`, `Medium` / `SemiBold`)
* **`p`**: Standard body copy (e.g. `13px` to `14px`, `Normal`)

### B. Shared Scale (`xs` to `2xl`)
A common scale aligned conceptually with Tailwind size steps:

* **`xs`**: Extra small (`11px` / `0.75rem`) → captions, fine print, indicators
* **`sm`**: Small (`12.5px` / `0.875rem`) → helper labels, secondary descriptions, badge items
* **`base` / `md`**: Medium / default (`14px` / `1rem`) → default body copy, form inputs, buttons
* **`lg`**: Large (`16px` / `1.125rem`) → list titles, primary control fields
* **`xl`**: Extra large (`18px` / `1.25rem`) → dialog headers, card titles
* **`2xl`**: 2x large (`20px` / `1.5rem`) → page subsection titles

---

## 3. Design Invariant: Theme-Aware Resolution in the SDK
To guarantee that typography responds dynamically to user changes (such as theme switching, dark mode, or scaling modifications):

* **No hardcoded sizes in SDK/look helpers**: helper methods and extensions must not define or return hardcoded float or pixel constants for standard semantic text.
* **Context resolution**: SDK/look methods must query the active `LumaTheme` context (native fallback) or `ShadcnLook` / `style.toml` (themed path) to determine size, line height, and weight.
* **Full style application**: the SDK typography system should resolve and apply **all three** of:
  * font size
  * line height
  * font weight

This is important because GPUI's built-in `text_sm()`-style helpers only set font size, while the SDK typography system must own the full typographic style.

---

## 4. Naming Rule: Avoid GPUI Method Collisions
GPUI already defines `Styled` helpers like:

- `text_xs()`
- `text_sm()`
- `text_base()`
- `text_lg()`
- `text_xl()`
- `text_2xl()`

Reusing those same names in SDK traits causes ambiguity and creates an API meaning conflict:

- GPUI `text_sm()` = fixed font size only
- SDK `text_sm()` = theme-resolved size + line height + weight

Therefore, **SDK-owned fluent helper names should not reuse GPUI's scale method names**.

### Preferred SDK naming style
Use a distinct SDK prefix style:

- `typography_xs()`
- `typography_sm()`
- `typography_md()`
- `typography_lg()`
- `typography_xl()`
- `typography_2xl()`

Semantic heading helpers are still fine because they do not collide with GPUI:

- `text_h1()`
- `text_h2()`
- `text_h3()`
- `text_h4()`
- `text_p()`

This gives us:

- clear ownership
- no trait ambiguity
- no prelude collision risk
- easy auditability of GPUI-vs-SDK typography usage

---

## 5. API Enhancements & Integration

### A. Theme Structures (`crates/sdk`)
`TextTokens` should support:

- semantic roles: `h1`, `h2`, `h3`, `h4`, `p`
- scale roles: `xs`, `sm`, `md`, `lg`, `xl`, `2xl`
- compatibility aliases for existing fields like:
  - `body`
  - `label`
  - `caption`
  - `title`
  - `code`

### B. Look Resolution (`crates/look-shadcn`)
`style.toml` should define typography entries, and `ShadcnLook` should expose resolvers such as:

```rust
pub fn typography_role(&self, role: ShadcnTextRole) -> LumaTextStyle;
pub fn typography_scale(&self, size: ShadcnTextSize) -> LumaTextStyle;
```

### C. SDK / Look Extension Trait Naming
The SDK-owned extension trait should avoid GPUI naming overlap:

```rust
pub trait LumaTypographyExt: Styled + Sized {
    // Semantic HTML-like headings
    fn text_h1(self) -> Self;
    fn text_h2(self) -> Self;
    fn text_h3(self) -> Self;
    fn text_h4(self) -> Self;
    fn text_p(self) -> Self;

    // SDK-owned scale names (preferred naming style)
    fn typography_xs(self) -> Self;
    fn typography_sm(self) -> Self;
    fn typography_md(self) -> Self;
    fn typography_lg(self) -> Self;
    fn typography_xl(self) -> Self;
    fn typography_2xl(self) -> Self;
}
```

These methods should apply full `LumaTextStyle` values, not just font-size.

### D. Optional Functional Helpers
For simple text rendering:

```rust
pub fn h1(text: impl Into<SharedString>) -> impl IntoElement;
pub fn h2(text: impl Into<SharedString>) -> impl IntoElement;
pub fn h3(text: impl Into<SharedString>) -> impl IntoElement;
pub fn p(text: impl Into<SharedString>) -> impl IntoElement;
```

### E. Optional Styling Closures
These are still useful ergonomics, but they are secondary to the typography policy itself:

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

---

## 6. Rule Set Summary

### Apps
- May use GPUI fixed helpers like `text_sm()` and `text_lg()`.
- May use SDK/theme typography when structural text should match the active look.
- Should avoid raw `.text_size(px(...))` for standard titles, labels, and paragraphs when a shared size helper is available.

### SDK / Look Layer
- Must not use GPUI `text_sm()` / `text_lg()` helpers for standard semantic text.
- Must not use hardcoded `.text_size(px(...))` for normal labels, titles, body text, or captions.
- Must use resolved `LumaTextStyle` / `LumaTypography` / `ShadcnLook` typography.
- Should prefer `typography_xs()`-style fluent method names for scale helpers.

### Special-Case Exceptions
Literal `.text_size(px(...))` remains acceptable for:

- icon glyph sizing
- debug readouts
- measured diagrams
- inspector overlays
- intentionally numeric visual annotations

---

## Tasks

### Phase 1: SDK Core Changes (`crates/sdk`)
- [ ] Ensure core text styles (`TextTokens` in [tokens.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/theme/tokens.rs)) support:
  - semantic roles (`h1`..`p`)
  - scale roles (`xs`..`2xl`)
  - compatibility aliases (`body`, `label`, `caption`, `title`, `code`)
- [ ] Define helper structures or baseline traits needed for SDK elements to scale size parameters dynamically.
- [ ] Establish the SDK rule that standard semantic text must resolve through `LumaTypography` / `LumaTextStyle`, not GPUI fixed scale helpers.

### Phase 2: Downstream Look-Shadcn Sizing & Extensions (`crates/look-shadcn`)
- [ ] Register typography rule mappings in the stylesheet resolver.
- [ ] Add typography configuration sections in `crates/look-shadcn/assets/style.toml`.
- [ ] Implement `ShadcnLook::typography_role(...)` and `ShadcnLook::typography_scale(...)`.
- [ ] Implement `LumaTypographyExt` using **non-colliding scale helper names** such as `typography_xs()`.
- [ ] Keep semantic helper names like `text_h1()` / `text_h2()` where they do not conflict with GPUI.
- [ ] Optionally implement `LumaStylerExt` and `ShadcnStylerExt` for inline/context styling ergonomics.
- [ ] Optionally add functional HTML-style element helpers (`h1`, `h2`, `h3`, `p`).

### Phase 3: Codebase Refactor & Cleanup
- [ ] Refactor gallery panes (especially `introduction`, `selection_panel`, `palette`, and pane headers) to replace hardcoded `.text_size(px(...))` with either:
  - GPUI fixed scale helpers in app code, or
  - theme-resolved typography when the text is structural and should track the active look.
- [ ] Refactor `luma-studio` panels (`dashboard`, `team`, `palette`, app shell) with the same app-level rule.
- [ ] Update SDK internal controls (like `field_label` in `controls/label.rs` and `navigation_sidebar/template.rs`) to use the new SDK typography tokens and resolvers.
- [ ] Audit SDK/look code to ensure GPUI `text_sm()`-style helpers are not used for semantic control text.

---

## Acceptance Criteria
- No hardcoded `.text_size(px(14.0))` or similar manual sizes are used for standard labels, titles, and paragraphs in the targeted gallery previews, workspace dashboard views, or SDK helpers listed above.
- SDK/look typography resolves through `LumaTypography` / `ShadcnLook` rather than GPUI fixed text scale helpers.
- App code may use GPUI `text_sm()`-style helpers, and those usages are easy to grep and audit later.
- SDK fluent scale helper naming avoids collision with GPUI `Styled` methods; preferred style is `typography_xs()` / `typography_sm()` / etc.
- Sizing changes dynamically when switching between themes or scale factors where the SDK/look typography path is used.
- The project builds cleanly with `cargo check` and runs without regression in text bounds or baseline alignment.

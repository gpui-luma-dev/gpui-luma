# Implementation Plan: Context-Aware Accordion & Decoupled Sidebar

This document describes the plan to refactor the Accordion control's custom content closure signature in the `gpui-luma` SDK to support GPUI context parameters, introduce a standard form row visual template, and clean up the Theme Studio sidebar's layout.

---

## 1. Goal Description

Currently, the `AccordionControl` in the SDK renders custom content using a context-less closure:
```rust
pub(crate) element: Option<Arc<dyn Fn() -> AnyElement + Send + Sync>>,
```
Because the closure does not receive the GPUI application context, any content containing stateful, interactive controls (like textfields and dropdowns in the Theme Studio sidebar) cannot reactively query active state handles without complex workarounds (like wrapping values in `Arc<RwLock>` locks). 

Additionally, the Theme Studio sidebar layout suffers from collapsing flexbox containers because it mixes state management logic with UI layouts and uses alignment properties (`align=start`) that prevent elements from stretching.

This plan details the steps to:
1. **Extend AccordionContent** with GPUI context parameters.
2. **Implement a standard `property_row` layout helper** in the SDK to solve horizontal shrinking bugs.
3. **Refactor Theme Studio Sidebar** by separating concerns into a `ThemeSidebarViewModel` and a clean, declarative `ThemeSidebar` View.
4. **Update the Gallery Pane** for the Accordion to exercise this context-aware rendering capability.

---

## 2. Proposed Changes

### Component A: SDK Accordion Control Refactoring

#### [MODIFY] [model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/accordion/model.rs)
- Update the signature of `AccordionContent::element` to accept references to the GPUI window and application context.
- Update constructors `new` and `custom` to match this signature:

```rust
pub struct AccordionContent {
    pub(crate) element: Option<Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>>,
}

impl AccordionContent {
    pub fn new(element: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        Self {
            element: Some(Arc::new(move |_, _| element.clone().into_any_element())),
        }
    }

    pub fn custom(custom: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        Self {
            element: Some(Arc::new(custom)),
        }
    }
}
```

#### [MODIFY] [template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/accordion/template.rs)
- Route the `window` and `cx` parameters into the `element` closure during rendering:

```rust
if let Some(renderer) = &item.content.element {
    content = content.child(renderer(window, cx));
}
```

---

### Component B: Standardizing Row Layout in SDK

#### [NEW] [form.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/form.rs) (or addition to a layout helper module)
Create a reusable layout template for property grids and form rows to ensure consistent sizing, margins, and automatic horizontal stretching:

```rust
use gpui::{Hsla, IntoElement, SharedString, div, prelude::*, px};
use crate::theme::LumaChrome;

/// Renders a standardized form row with a label, optional color swatch, and input control.
pub fn property_row(
    label: impl Into<SharedString>,
    swatch_color: Option<Hsla>,
    control: impl IntoElement,
    chrome: &LumaChrome,
) -> impl IntoElement {
    vstack! {
        gap=4; // Default cross-alignment is stretch - forces children to take full width
        div()
            .text_size(px(11.0))
            .text_color(chrome.muted_text)
            .child(label.into()),
        hstack! {
            gap=8 align=center;
            swatch_color.map(|color| {
                div()
                    .size(px(16.0))
                    .flex_shrink_0()
                    .rounded(px(4.0))
                    .bg(color)
                    .border_1()
                    .border_color(chrome.border)
            }),
            div()
                .flex_1()
                .min_w(px(0.0))
                .child(control),
        },
    }
}
```

---

### Component C: Refactoring Theme Studio Sidebar

#### [MODIFY] [theme_sidebar.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/theme_sidebar.rs)
- Extract state management, active selectors, hex textfield tables, and subscription maps into a separate `ThemeSidebarViewModel` entity.
- Make the `ThemeSidebar` View completely stateless. In `ThemeSidebar::render`:
  1. Initialize `ScrollContainer` to handle styled scrollbars.
  2. Read the active selector and accordion from the view model.
  3. Render the accordion content reactively using the new context-aware `(window, cx)` closures.
  4. Render the categories using the new stateless `property_row` helper, completely removing the legacy `Arc<RwLock>` synchronization state.
  5. Ensure the viewport content `body` has `.w_full()` to take the full width of the scroll container.
  6. Enable `.full_width(true)` on the `TextField` builders.

---

### Component D: Gallery Accordion Update

#### [MODIFY] [pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/accordion/pane.rs)
- Update `demo_item` content closures to accept `_window` and `_cx` parameters.
- Add a new **Interactive Form Sample** within the pane that renders a stateful `TextField` and dynamically reads its value on each frame inside the accordion, verifying context-awareness and live reactivity.

---

## 3. Verification Plan

### Automated Verification
- Run compilation checks on the workspace:
  ```bash
  cargo check
  ```
- Run unit tests to verify standard controls behavior:
  ```bash
  cargo test
  ```


# Proposal: DockPanel Layout System for GPUI-Luma

This proposal outlines the layout composition issues in `gpui-luma` panels and defines a WPF-inspired `DockPanel` system (builder and macro) designed to make visual layout structure self-documenting and easy to read.

---

## 1. The Problem

Building multi-pane widgets and panels in GPUI currently introduces layout code noise:

1. **Flexbox boilerplate:** Setting up standard layouts (like a header pinned to the top, a footer pinned to the bottom, and a body filling the rest) requires verbose nested `.flex().flex_col().h_full()` structures combined with `.flex_1().h(px(0.0))` hacks to manage scrolling and overflow clipping.
2. **"How" vs. "What" (Geometry-Oriented Styling):** Elements that represent basic visual placements are written in CSS-like geometry details, which are hard for developers to read and error-prone for LLMs to generate.
3. **Lack of Clear Intent:** Standard stacks (`vstack!`, `hstack!`) do not explicitly convey boundaries or edge-docking relationships.

---

## 2. The Solution: WPF-Style `DockPanel`

We propose introducing a classic `DockPanel` layout primitive (implemented builder-first with macro sugar) inspired by WPF/XAML. 

A `DockPanel` arranges children by docking them to the outer boundaries (Top, Bottom, Left, Right) and letting the last/designated child fill all remaining space.

This design ensures:
* **IDE Auto-complete & Type Checking:** The builder API uses type-safe layout methods.
* **Self-Documenting Code:** The code directly states which components sit on the edges and which fills the center.
* **No Flex Hacks:** The layout logic handles size constraints and overflow clipping under the hood.

### A. The Builder API
```rust
DockPanel::new()
    .top(render_header())
    .bottom(render_footer())
    .fill(render_body())
    .into_element()
```

### B. The Macro Sugar
```rust
dock_panel! {
    top: render_header(),
    bottom: render_footer(),
    fill: render_body()
}
```

---

## 3. Example Usage: Card Composition

Here is how a standard panel containing a header, footer, and filling body is structured using the `DockPanel` system:

```rust
impl gpui::Render for ExamplePaneState {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.look
            .card("example-card")
            .child_render(move |_, _| {
                dock_panel! {
                    // Header pinned to the top
                    top: vstack! {
                        gap=4;
                        div().text_size(px(16.0)).child("Pane Title"),
                        div().text_size(px(12.0)).child("Description of panel details."),
                    },
                    
                    // Action controls pinned to the bottom
                    bottom: hstack! {
                        gap=8;
                        align_right! {
                            hstack! {
                                gap=8;
                                &self.cancel_button,
                                &self.submit_button,
                            }
                        }
                    },
                    
                    // Main workspace filling the center and handling scroll overflow
                    fill: scrollable! {
                        vstack! {
                            gap=12;
                            &self.content_field_1,
                            &self.content_field_2,
                            &self.content_field_3,
                        }
                    }
                }
                .into_any_element()
            })
            .render(window, cx)
    }
}
```

---

## 4. Technical Implementation Details

### A. Flexbox Mapping Under the Hood
The `DockPanel` builder compiles down to a nested flex hierarchy. For a standard top-bottom-fill setup, it generates:

```rust
// Macro/Builder expansion
::gpui::div()
    .flex()
    .flex_col()
    .h_full()
    .child(top_element) // Pinned top
    .child(
        ::gpui::div()
            .flex_1()
            .h(::gpui::px(0.0)) // Forces Taffy flex engine to contain scrollable height
            .child(fill_element)
    )
    .child(bottom_element) // Pinned bottom
```

### B. Children Type Resolution
To accept entities, references, and raw elements interchangeably:
```rust
pub trait LumaLayoutElement {
    fn resolve_element(self, cx: &mut AppContext) -> gpui::AnyElement;
}
```

---

## 5. Gallery Documentation Pane

To build out layout experience and provide a reference for developers and LLMs, we will add a dedicated **DockPanel Gallery Pane** under `apps/gallery/src/gallery/panes/dock_panel/`.

* **Visual Sandbox:** Shows how changing edge alignments and resizing targets dynamically shifts boundaries.
* **WPF-Style Complete Demo:** A dedicated layout scenario demonstrating all 5 dock boundaries simultaneously, mirroring standard desktop layouts:
  ```rust
  dock_panel! {
      top: render_menu(),
      bottom: render_status_bar(),
      left: render_left_navigation().w(px(120.0)),
      right: render_right_sidebar().w(px(100.0)),
      fill: render_main_content()
  }
  ```
* **Code Reference:** Serves as the codebase authority on how to construct and use the `DockPanel` builders and macros cleanly.


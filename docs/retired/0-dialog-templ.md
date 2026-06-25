# Dialog Templating & Composition Specification

This document details the pure composition pattern for the `Dialog` control, leveraging `OverlayWindow` internally and decoupling visual structuring into polymorphic templates.

## Architectural Guidelines

1. **No Builder Anti-Pattern**: The `Dialog` control is spawned first as a live `Entity<Dialog>`. Configuration functions (like setting the title, content, or callbacks) are called directly on the control rather than using a pre-spawn builder.
2. **Reuse `OverlayWindow`**: The `Dialog` control uses `OverlayWindow` internally to manage focus trapping, modal backdrop, positioning, and dragging.
3. **Decoupled Layout (Templates)**: Inside the dialog's content callback, rendering delegates layout and style composition entirely to a `DialogTemplate` implementation rather than using hardcoded margins, spacing, or DockPanels directly in the control.

---

## 1. The `DialogTemplate` Trait

The template defines how the logical sections of a dialog (e.g., title, header actions, body, footer) are visually laid out and spaced.

```rust
pub(in crate::gallery) trait DialogTemplate: Send + Sync {
    fn render(
        &self,
        model: &DialogRenderModel,
        overlay_model: &OverlayWindowRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;
}
```

---

## 2. Dialog View Integration

The `Dialog` control maintains its structural slots and delegates layout composition to the active template in its internal rendering hook.

```rust
pub struct Dialog {
    overlay: OverlayWindow,
    model: DialogModel,
    template: Arc<dyn DialogTemplate>,
}

impl Dialog {
    pub fn spawn(
        look: Arc<ShadcnLook>,
        id: impl Into<SharedString>,
        template: Arc<dyn DialogTemplate>,
        cx: &mut Context<Self>,
    ) -> Self {
        let dialog = cx.entity().clone();
        
        let overlay = look
            .overlay_window(id)
            .content(move |overlay_model, window, app| {
                // Fetch the template and render state dynamically
                let (model, template) = {
                    let d = dialog.read(app);
                    (d.model.clone(), Arc::clone(&d.template))
                };
                template.render(&model, overlay_model, window, app)
            })
            .spawn(cx);

        Self { overlay, model: DialogModel::default(), template }
    }
}
```

---

## 3. Themed Templates (e.g., Shadcn Look)

Design system layouts are implemented inside themed template structures (e.g. `DefaultDialogTemplate` or `ShadcnDialogTemplate`), keeping layout boundaries like padding scales, dividers, and alignments separate from control logic.

```rust
impl DialogTemplate for DefaultDialogTemplate {
    fn render(
        &self,
        model: &DialogRenderModel,
        overlay_model: &OverlayWindowRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let mut content = div().w_full().min_w_0().flex().flex_col().gap(px(14.0));

        // Layout the header
        if model.title.is_some() || model.header_end.is_some() {
            let mut header = DockPanel::new().last_child_fill(true);

            if let Some(header_end) = &model.header_end {
                header = header.right(header_end(overlay_model, window, cx));
            }

            if let Some(title) = &model.title {
                header = header.fill(div().w_full().min_w_0().text_xl().child(title.clone()));
            } else {
                header = header.fill(div().w_full().min_w_0());
            }

            content = content.child(header);
        }

        // Layout the body
        if let Some(body) = &model.body {
            content = content.child(div().w_full().min_w_0().child(body(overlay_model, window, cx)));
        }

        // Layout the footer
        if let Some(footer) = &model.footer {
            content = content.child(div().w_full().min_w_0().pt(px(4.0)).child(footer(overlay_model, window, cx)));
        }

        content.into_any_element()
    }
}
```

# Unified Control Architecture: The Presenter Pattern

This document outlines the second-generation control architecture for the GPUI-Luma SDK. It formalizes a **Lookless Presenter Pattern** that decouples interaction logic, business data, and visual presentation.

---

## 1. Core Philosophy: Separation of Concerns

The architecture splits a single control (e.g., `Button`) into three distinct layers:

| Layer | Responsibility | Implementation Reference |
| :--- | :--- | :--- |
| **Logic** | Event handling, focus, and pointer state. | `CommandCore`, `RangeCore`, etc. |
| **Model** | Business data and configuration state. | `ButtonModel<D>`, `SliderModel<D>`. |
| **Presenter** | Mapping model state to visual sub-elements. | `ControlContent<M>` (The "Face"). |
| **Template** | Composing the final UI and applying themes. | `ButtonTemplate<D>` (The "Engine"). |

By making the control generic over a data type `D`, we allow the control to host any state without the SDK needing to know its structure.

---

## 2. Infrastructure: `template.rs`

The engine driving the "Lookless" rendering is defined in `crates/sdk/src/controls/template.rs`. It provides a generic pipeline for applying style overrides.

### 2.1 The Modifier Pipeline
Styles are applied via a **Modifier** pipeline. A `Modifier<M>` is a boxed closure that takes a `Stateful<Div>` and the Render Model `&M`.

```rust
pub type Modifier<M> = Box<dyn Fn(Stateful<Div>, &M) -> Stateful<Div> + Send + Sync + 'static>;
```

### 2.2 Orchestration Traits
Templates implement `TemplateWithModifiers<M>`, which provides the `apply_modifiers` helper. This ensures a consistent order of operations:
1. Template resolves the base theme.
2. Template constructs the base element.
3. Template executes `self.apply_modifiers(element, model)` to run user-defined tweaks.

### 2.3 The `ControlTemplate` Struct
Instead of repeating code, most controls use the generic `ControlTemplate<T, M>` struct. It handles the storage of the theme `Arc<T>` and the `Vec<Modifier<M>>`.

### 2.4 Macro-Driven Definitions
To keep the SDK code clean, we use the `define_control_template!` macro. This generates the necessary type aliases and default instance helpers:

```rust
define_control_template!(
    ButtonTemplate,            // Alias name
    dyn ButtonFamilyTheme,     // Theme trait
    ButtonRenderModel,         // Model type
    IButtonTemplate,           // Target trait
    default_button_family_theme() // Default theme provider
);
```

---

## 3. Usage Cookbook (Examples)

### 3.1 Simple Text Button
```rust
Button::new("Save Changes")
    .kind(ButtonKind::Prominent)
    .spawn(cx)
```

### 3.2 Circular Icon Button
```rust
Button::icon("delete-btn", LucideIcon::Trash)
    .kind(ButtonKind::Ghost)
    .spawn(cx)
```

### 3.3 Hybrid Layout (Text + Icon)
```rust
Button::new("settings")
    .content(|_, _| {
        div().flex().gap_2()
            .child(render_icon(LucideIcon::Settings))
            .child("Settings")
    })
    .spawn(cx)
```

### 3.4 One-off Style Modifier
Applying a specific visual tweak without creating a new template.
```rust
let danger_template = default_button_template()
    .with_modifier(|el, model| {
        el.bg(rgb(0xdc2626)).text_color(rgb(0xffffff))
    });

Button::new("Delete Account")
    .template(danger_template)
    .spawn(cx)
```

### 3.5 Reactive Data Update
```rust
// Spawning with data
let btn = Button::new("sync").data(SyncState::Idle).spawn(cx);

// Updating elsewhere
btn.update(cx, |btn, cx| {
    btn.set_data(SyncState::Syncing(0.45), cx);
});
```

---

## 4. Beyond Buttons: Scaling the Architecture

The power of this architecture is its consistency. To build a non-button control (e.g., a `Slider`), we simply swap the **Logic Core** and **Theme Resolver**.

### 4.1 Example: The Reactive Slider
A `Slider` follows the exact same pattern but uses `RangeCore` for logic and a custom `SliderRenderModel`.

```rust
pub struct Slider<D = ()> {
    model: SliderModel<D>,
    core: RangeCore, // Specialized logic core
}

// User Usage
Slider::new("volume")
    .min(0.0).max(100.0)
    .data(AudioState { volume: 50.0 })
    .content(|model, _| {
        // Slider uses the presenter to render a dynamic tooltip!
        div().child(format!("Vol: {}%", model.data.volume))
    })
    .spawn(cx);
```

---

## 5. The Presenter Pattern (`ControlContent<M>`)

The breakthrough of this architecture is moving away from hardcoded content fields to a closure-based **Presenter**:

```rust
pub type ControlContent<M> = Arc<dyn Fn(&M, &mut App) -> AnyElement + Send + Sync>;
```

### Benefits:
- **Dynamic Layouts**: The button face can change based on hover, focus, or custom data.
- **Lazy Rendering**: Complex content is only built when the button actually renders.
- **Unified API**: One button type handles text, icons, images, or custom flex layouts.

---

## 6. The Fluent API (`HasContent`)

To keep the API ergonomic ("Swift-ier"), we use a trait-based fluent builder:

```rust
pub trait HasContent<M> {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.set_content(Arc::new(move |_, _| {
            div().child(label.clone()).into_any_element()
        }));
        self
    }

    fn content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&M, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement;
}
```

---

## 7. Reactive State Management Deep Dive

By parameterizing the control as `Button<D>`, we enable first-class reactivity. 

```rust
// 1. Define custom state
struct UploadState { progress: f32 }

// 2. Spawn a reactive button
let btn = Button::new("upload")
    .data(UploadState { progress: 0.0 })
    .content(|model, _| {
        div().child(format!("{}%", (model.data.progress * 100.0) as u32))
    })
    .spawn(cx);

// 3. Update state reactively
btn.update(cx, |btn, cx| {
    btn.set_data(UploadState { progress: 0.5 }, cx);
});
```

---

## 8. Two-way Signaling (Interior Mutability)

Because modifiers run *inside* the template rendering pass, they sometimes need to communicate style changes back to the layout engine (e.g., overriding the focus ring radius). 

We solve this using **Interior Mutability** (e.g., `Cell` or `RefCell`) in the Render Model. Since modifiers receive a shared reference `&M`, they can still mutate these signaling fields.

---

## 9. One Small Polish for the Future

As you scale to those 40+ controls, you might find that certain **Modifier Sets** are used repeatedly (e.g., a `danger_zone` style that applies to Buttons, Sliders, and Checkboxes). 

You can eventually support **"Modifier Groups"** or **"Style Bundles"** that allow a single `.with_style(DangerStyle)` call to apply multiple modifiers at once. This keeps the call sites clean while centralizing high-level visual themes across diverse control types.

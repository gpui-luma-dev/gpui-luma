# GPUI-Luma Templates v3: Slot-Bound Templates with Internal Look

This document defines v3 of template customization for GPUI-Luma.

v3 replaces "public look structs as the prominent customization surface" with a binding-style slot model inspired by WPF concepts (`TemplateBinding`, resource lookup), while preserving Rust ergonomics and type safety.

---

## Why v3

In v2, parameterization improved template customization, but exposed a boundary problem:

- `Theme` risked becoming too control-specific.
- `Look` risked becoming an external API contract.
- Callers wanting one-off dramatic styling (e.g. emergency button) needed low-level internals.
- New theme authors could feel pressure to provide control-specific look behavior.

v3 resolves this by introducing **Template Slots** and a strict layering model.

---

## Design Goals

1. Keep `theme` focused on **themey primitives** (tokens, semantic roles, mode).
2. Keep control-level rendering internals (`Look`) **internal**.
3. Enable caller overrides without exposing internals as public API.
4. Preserve "easy escape hatch": custom templates remain first-class.
5. Support runtime tooling/introspection via metadata registries.

---

## Core Model

v3 standardizes these layers:

1. **Theme Tokens (public)**  
   Global palette/typography/metric primitives and semantic roles.

2. **Template Slots (public, typed)**  
   Stable named values templates bind to at render time.

3. **Look (internal)**  
   Ephemeral computed render snapshot used by template implementation only.

4. **Template Implementation (public trait, internal defaults)**  
   Default template resolves slots and renders.
   Custom templates can ignore defaults entirely.

---

## Template Slots (Binding-Style)

A **slot** is a typed style key consumed by a template.

Examples for `Button`:

- `button.background.default`
- `button.background.hovered`
- `button.background.pressed`
- `button.background.disabled`
- `button.foreground.default`
- `button.border.color`
- `button.border.width`
- `button.radius`
- `button.height`
- `button.padding.x`
- `button.padding.y`
- `button.gap`
- `button.focus_ring.color`
- `button.focus_ring.enabled`

Templates bind to slots; they do not directly expose internal look fields.

### Typed Slots

Slots should be represented with typed enums/newtypes rather than raw strings in core rendering code.

Conceptual categories:

- `Color`
- `Pixels`
- `Bool`
- `Number`
- `FontWeight`
- `FontSize`
- `Icon`

---

## Resolution Precedence

When a template asks for a slot value, resolver precedence is:

1. **Explicit template override set**
2. **Named style preset** (e.g. `"emergency"`)
3. **Theme semantic mapping**
4. **SDK default fallback**

This gives WPF-like behavior while preserving predictability.

---

## Public API Direction

### Theme API (public)

Theme remains token/semantic-focused:

- palette/tokens
- typography/metric scale
- semantic roles (prominent/standard/ghost/etc)
- mode (light/dark)

Theme does **not** require callers to construct control-specific `Look`.

### Template API (public)

Templates expose:

- constructor with theme/tokens
- optional slot overrides/preset hook
- trait-based render entrypoint

### Look API (internal)

Control look structs are implementation detail:

- may remain in codebase for clarity/perf
- not intended as prominent external customization contract
- can evolve without broad downstream breakage

---

## Parameterization in v3

v3 retains template parameterization but reframes it:

- **Structural params**: layout/composition policy (gaps, offsets, icon placement policy)
- **Slot overrides**: visual values and state mappings
- **No requirement** to expose raw look patch structs publicly

For callers:

- Normal customization: choose theme + preset + small override map.
- Dramatic changes: write a custom template (recommended escape hatch).

---

## Introspection Metadata

v3 adds slot metadata alongside existing theme usage metadata.

### New metadata types

- `TemplateSlotUsage`
- `TemplateSlotMetadata`
- `TemplateSlotType`
- `all_template_slot_usages()`

This enables tooling/property editors to discover:

- which slots a template consumes
- expected type/range
- state semantics
- documentation hints

---

## Migration Plan from v2

### Step 1: Introduce slots for one control (`Button`)
- Define typed slot keys.
- Update default button template to resolve slots.
- Keep old look path internally for transition.

### Step 2: Introduce resolver stack
- Overrides -> preset -> theme mapping -> defaults.

### Step 3: Deprecate direct look customization
- Keep internal structs.
- Remove/avoid new public APIs that patch raw look fields.

### Step 4: Expand to complex controls
- `NavigationSidebar`, `PopupMenu`, `ContextMenu`, `NavView`.

### Step 5: Add slot usage registry
- Expose metadata for gallery tooling.

---

## Decision Rules (v3)

For any new value, place it by intent:

- **Theme primitive/semantic?** -> Theme tokens/roles.
- **Control render value consumed by template?** -> Template Slot.
- **Pure layout policy?** -> Template structural params.
- **One-off dramatic redesign?** -> Custom template.

---

## Example: Emergency Button

In v3, emergency styling should not require patching internals.

Preferred approaches:

1. Add semantic preset/key (`"emergency"`) resolved to button slots.
2. Optionally support targeted slot overrides for that button instance.
3. If behavior/structure is radically different, supply custom template.

---

## Non-Goals

- Recreating full WPF dependency property system.
- Making every internal render value publicly mutable.
- Eliminating default templates in favor of only custom templates.

---

## Summary

v3 defines a cleaner contract:

- Theme is themey.
- Templates bind to typed slots.
- Look is internal.
- Parameterization remains, but with stable boundaries.
- Escape hatch remains simple: write your own template with SDK source as guide.

---

## Final Section: End-to-End App Example (Button Template Parameter Customization)

```rust
use std::sync::Arc;

use gpui::{Context, Entity, Hsla};

use luma_sdk::controls::button::{
    Button,
    ButtonEvent,
    ButtonKind,
    ButtonTemplate,
    ButtonTemplateParams,
    ThemedButtonTemplate,
};
use luma_sdk::theme::default_button_family_theme;

const EMERGENCY_DISABLED_OPACITY: f32 = 0.72;
const EMERGENCY_RADIUS: f32 = 10.0;
const EMERGENCY_BACKGROUND: Hsla = Hsla { h: 0.00, s: 0.83, l: 0.48, a: 1.0 };

fn emergency_button_template() -> Arc<dyn ButtonTemplate> {
    let params = ButtonTemplateParams {
        disabled_opacity: EMERGENCY_DISABLED_OPACITY,
        radius: EMERGENCY_RADIUS,
        background_default: EMERGENCY_BACKGROUND,
        ..Default::default()
    };

    Arc::new(ThemedButtonTemplate::new(default_button_family_theme()).with_params(params))
}

pub fn spawn_emergency_button(cx: &mut Context<MyView>) -> Entity<Button> {
    Button::new("shutdown-now")
        .label("Emergency Shutdown")
        .kind(ButtonKind::Prominent)
        .template(emergency_button_template())
        .spawn(cx)
}

pub fn wire_emergency_button(button: &Entity<Button>, cx: &mut Context<MyView>) {
    cx.subscribe(button, |this, _button, event: &ButtonEvent, cx| {
        if matches!(event, ButtonEvent::Click) {
            this.open_shutdown_confirmation_dialog(cx);
        }
    })
    .detach();
}

struct MyView;

impl MyView {
    fn open_shutdown_confirmation_dialog(&mut self, _cx: &mut Context<Self>) {}
}
```
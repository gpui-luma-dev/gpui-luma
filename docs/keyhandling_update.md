# Key Handling Update Design

## 1. Purpose

This document proposes the next keyhandling shape for GPUI-Luma controls.
It addresses a recurring design problem in `keyhandling.rs`: contexts are
currently public string constants, and new controls can accidentally inherit
control-specific names or semantics that only partly fit.

The immediate example is `Slider` and `Scrollbar`. Both use generic value
actions such as `IncreaseValue`, `DecreaseValue`, `MoveToStart`, and
`MoveToEnd`, but their keyboard behavior is not identical:

- `Slider` is a range value input. `Right` and `Up` increase, `Left` and
  `Down` decrease.
- `Scrollbar` is a scroll offset input. `Right` and `Down` increase, `Left`
  and `Up` decrease.

The fix is not to create more ad hoc constants. The SDK should introduce
behavior-level key profiles and let contexts become an implementation detail
of those profiles.

## 2. Design Goals

- Context names describe keyboard behavior, not control type.
- Controls choose a key profile, not a raw string context.
- `default_control_key_bindings` is assembled from named profiles.
- Adding a new control requires an explicit choice: reuse an existing profile
  or add a new profile with documented behavior.
- Host applications can still bind the default keys with one call.
- Advanced applications can inspect or bind profile-specific keys.
- Existing typed actions remain shared and semantic.

Non-goals:

- Do not build a full user-configurable keymap registry yet.
- Do not move focus traversal into control keyhandling.
- Do not encode every control state as a separate key context.
- Do not add runtime string normalization or string-to-profile parsing.

## 3. Core Concept

Introduce a public profile enum:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ControlKeyProfile {
    Command,
    Choice,
    RadioGroup,
    RangeValue,
    ScrollOffset,
    Menu,
    ContextMenu,
}
```

A profile owns:

- the GPUI key context string,
- the default `KeyBinding` rows for that behavior,
- the intended semantic contract for controls using it.

Controls should call `ControlKeyProfile::context()` through a small helper
instead of importing raw context constants directly.

```rust
impl ControlKeyProfile {
    pub const fn context(self) -> &'static str {
        match self {
            Self::Command => "LumaCommandControl",
            Self::Choice => "LumaChoiceControl",
            Self::RadioGroup => "LumaRadioGroup",
            Self::RangeValue => "LumaRangeValue",
            Self::ScrollOffset => "LumaScrollOffset",
            Self::Menu => "LumaMenuControl",
            Self::ContextMenu => "LumaContextMenuControl",
        }
    }
}
```

The string is still public and stable through the profile API, but the primary
SDK vocabulary becomes behavior profiles.

## 4. Public API Shape

`crates/sdk/src/keyhandling.rs` should expose:

```rust
pub enum ControlKeyProfile {
    Command,
    Choice,
    RadioGroup,
    RangeValue,
    ScrollOffset,
    Menu,
    ContextMenu,
}

impl ControlKeyProfile {
    pub const fn context(self) -> &'static str;
    pub fn default_bindings(self) -> Vec<KeyBinding>;
}

pub fn default_control_key_bindings() -> Vec<KeyBinding>;
pub fn bind_default_control_keys(cx: &mut App);
```

The raw constants can remain only if there is a clear need for direct GPUI
integration. If retained, they should be secondary exports and their names
must match profile behavior:

```rust
pub const LUMA_RANGE_VALUE_CONTEXT: &str = "LumaRangeValue";
pub const LUMA_SCROLL_OFFSET_CONTEXT: &str = "LumaScrollOffset";
```

Do not reintroduce control-type aliases such as `LUMA_SLIDER_CONTEXT` or
`LUMA_SCROLLBAR_CONTEXT`.

## 5. Default Binding Assembly

`default_control_key_bindings` should become a profile composition rather than
one long literal table.

```rust
pub fn default_control_key_bindings() -> Vec<KeyBinding> {
    [
        ControlKeyProfile::Command,
        ControlKeyProfile::Choice,
        ControlKeyProfile::RadioGroup,
        ControlKeyProfile::RangeValue,
        ControlKeyProfile::ScrollOffset,
        ControlKeyProfile::Menu,
        ControlKeyProfile::ContextMenu,
    ]
    .into_iter()
    .flat_map(ControlKeyProfile::default_bindings)
    .collect()
}
```

This keeps the default set easy to scan and makes each behavior easier to test
in isolation.

Profile-specific methods should keep binding rows close to their behavior:

```rust
impl ControlKeyProfile {
    pub fn default_bindings(self) -> Vec<KeyBinding> {
        let context = self.context();

        match self {
            Self::RangeValue => vec![
                KeyBinding::new("left", DecreaseValue, Some(context)),
                KeyBinding::new("down", DecreaseValue, Some(context)),
                KeyBinding::new("right", IncreaseValue, Some(context)),
                KeyBinding::new("up", IncreaseValue, Some(context)),
                KeyBinding::new("pagedown", DecreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", IncreaseValueLarge, Some(context)),
                KeyBinding::new("home", MoveToStart, Some(context)),
                KeyBinding::new("end", MoveToEnd, Some(context)),
            ],
            Self::ScrollOffset => vec![
                KeyBinding::new("left", DecreaseValue, Some(context)),
                KeyBinding::new("up", DecreaseValue, Some(context)),
                KeyBinding::new("right", IncreaseValue, Some(context)),
                KeyBinding::new("down", IncreaseValue, Some(context)),
                KeyBinding::new("pageup", DecreaseValueLarge, Some(context)),
                KeyBinding::new("pagedown", IncreaseValueLarge, Some(context)),
                KeyBinding::new("home", MoveToStart, Some(context)),
                KeyBinding::new("end", MoveToEnd, Some(context)),
            ],
            _ => todo!("other profiles follow the same shape"),
        }
    }
}
```

## 6. Control Usage

Controls should attach profiles at the focus-tracked element.

```rust
use crate::keyhandling::ControlKeyProfile;

template
    .render(&model, handlers, window, cx)
    .track_focus(self.interaction.focus_handle())
    .key_context(ControlKeyProfile::RangeValue.context())
```

Expected current mappings:

- `Button`, `IconButton`: `Command`.
- `ToggleButton`: `Command`.
- `Checkbox`, `Switch`: `Choice`.
- `RadioGroup`: `RadioGroup`.
- `Slider`: `RangeValue`.
- `Scrollbar`: `ScrollOffset`.
- `DropdownMenu`: `Menu`.
- `ContextMenu`: `ContextMenu`.

## 7. Adding Future Controls

When adding a control, decide the profile before writing bindings.

Use an existing profile when all of these are true:

- the same physical keys should dispatch the same actions,
- the same action names carry the same semantic meaning,
- the same directionality applies,
- disabled behavior and event emission can follow the profile contract.

Add a new profile when any of these are true:

- a key has a different direction or meaning,
- the control has an established keyboard convention that differs from existing
  profiles,
- a profile name would make the control’s behavior harder to explain,
- adding the control would require branching inside unrelated controls.

Likely future profiles:

- `VerticalValue`: spin buttons or vertical range inputs where `Up` increases
  and `Down` decreases, but horizontal arrows may be unused.
- `TabList`: roving tab selection where arrows move between tabs.
- `GridNavigation`: two-dimensional composite navigation.
- `TextInput`: text editing and composition, likely mostly raw-key or
  platform-owned.

## 8. Tests

Add focused tests for profile bindings:

- each profile returns parseable bindings,
- each profile has the expected binding count,
- `RangeValue` and `ScrollOffset` intentionally differ for `Up`, `Down`,
  `PageUp`, and `PageDown`,
- `default_control_key_bindings` equals the concatenation of all default
  profile bindings.

Avoid tests that only assert one global count unless they also identify which
profile changed. Global-count-only tests make legitimate profile edits harder
to review.

## 9. Migration Plan

1. Add `ControlKeyProfile` and move context strings behind `context()`.
2. Implement `default_bindings()` per profile.
3. Rebuild `default_control_key_bindings()` from the profile list.
4. Update existing controls to call `ControlKeyProfile::<Profile>.context()`.
5. Remove direct use of raw context constants from control modules.
6. Keep behavior-named constants only if needed by downstream app code.
7. Update `docs/keyhandling.md` after the implementation lands.

## 10. Open Questions

- Should behavior-named raw constants remain public, or should `context()` be
  the only public path?
- Should host apps be able to bind only selected profiles through a helper such
  as `bind_control_key_profiles(cx, profiles)`?
- Should profiles expose a stable display name for documentation or debug UIs?
- Should profile binding vectors be `Vec<KeyBinding>` or an iterator-like
  helper to avoid allocation before `cx.bind_keys`?

The recommendation is to start with the enum plus `context()` and
`default_bindings()`. That is enough to prevent recurring naming mistakes
without overcommitting the SDK to a large keymap framework.

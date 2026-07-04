# Radix / Shadcn Size and Radius Support Catalog

This document details the scoping of `size` and `radius` properties across Radix Themes and Shadcn UI components. Use this guide to ensure correct translation and alignment of styles in `look-shadcn`.

---

## 1. Components Supporting Both Explicit `size` and `radius` Overrides

These controls allow individual overrides for both size (density/height) and radius (corner rounding) to bypass global theme values:

*   **Button / IconButton**
    *   **Radix Specs**: `size` (`"1" | "2" | "3" | "4"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project File**: [button.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/button.rs)
*   **TextField / TextArea**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project Files**: [textfield.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/textfield.rs), [textarea.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/textarea.rs)
*   **Switch**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`, defaults to `"full"`)
    *   **Project File**: [switch.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/switch.rs)
*   **Slider**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project File**: [slider.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/slider.rs)
*   **Progress**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project File**: [progress.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/progress.rs)
*   **SegmentedControl**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project File**: [tabs_navigation.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/tabs_navigation.rs)
*   **Select (Trigger Component)**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project File**: [selector.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/selector.rs)
*   **Badge / Avatar**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"` for Badge, `"1"` to `"9"` for Avatar), `radius` (`"none" | "small" | "medium" | "large" | "full"`)
    *   **Project File**: [badge.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/elements/badge.rs)

---

## 2. Components Supporting `size` But Inheriting/Fixed Radius

These controls scale on density/size but either clamp the radius offsets or hardcode it to avoid visual degradation:

*   **Checkbox / CheckboxGroup**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`). No custom `radius` property.
    *   **Radius Behavior**: Inherits theme radius but clamps/offsets to prevent complete circular/pill rounding.
    *   **Project File**: [checkbox.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/checkbox.rs)
*   **Radio / RadioGroup**
    *   **Radix Specs**: `size` (`"1" | "2" | "3"`). No custom `radius` property.
    *   **Radius Behavior**: Fixed to `"full"` (circular).
    *   **Project File**: [radio.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/radio.rs)

---

## 3. Container / Panel Components (Inheriting Global Theme Radius)

Containers, dialogs, and popups do not expose individual `radius` overrides to maintain layout consistency. However, they inherit the global theme radius:

*   **Card**
    *   **Radix Specs**: `size` (`"1" | "5"` controls layout padding). Inherits global `radius` (`Lg`).
    *   **Project File**: [card.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/card.rs)
*   **Dialog / AlertDialog**
    *   **Radix Specs**: `size` (`"1" | "4"` controls container width). Inherits global `radius` (`Xl` / `Lg`).
    *   **Project File**: [overlay_window.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/overlay_window.rs)
*   **Popover / DropdownMenu / ContextMenu**
    *   **Radix Specs**: Sizing handles layout bounds. Inherits global `radius` (`Lg` for container, `Sm` for items).
    *   **Project Files**: [popup_menu.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/popup_menu.rs), [floating_menu.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/floating_menu.rs)

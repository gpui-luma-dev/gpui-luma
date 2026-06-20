# Implementation Plan: Exhaustive Scaffolding of Shadcn Control Looks

To achieve full visual parity with `gpui-luma-theme-radix` and ensure that downstream applications (`theme-studio` and `gallery`) can compile with `ShadcnLook` templates, the `crates/look-shadcn` crate must implement the complete set of 31 control look files.

---

## Exhaustive Proposed Changes

We will introduce the following files into [crates/look-shadcn/src/](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src):

### 1. SDK Component Mapping Files (31 Files)

Each file maps `ShadcnLook` parsed CSS variables and layout configurations to the specific control palettes in `gpui_luma::controls`:

| File | Target Mapping | Purpose |
| :--- | :--- | :--- |
| **`accordion.rs`** | `AccordionPalette` & Trigger/Content | Styles expand/collapse sections. |
| **`action.rs`** | Style Token Mapping | Translates `ShadcnButtonStyle` to semantic token pairs. |
| **`autocomplete.rs`** | `AutocompleteTextBoxLook` | Styles search selection inputs. |
| **`button.rs`** | `ButtonFamilyPalette` & Look | Styles primary, secondary, outline, ghost, and destructive buttons. |
| **`checkbox.rs`** | `CheckboxPalette` | Styles checked, unchecked, and disabled checkboxes. |
| **`context_menu.rs`** | `ContextMenuLook` | Styles desktop-style context popup overlays. |
| **`control_group.rs`** | `ControlGroupListLook` | Styles grouped buttons and selections. |
| **`controls.rs`** | `ShadcnLookControlExt` | Builder extension traits matching `RadixThemeControlExt` (e.g., `look.primary_button("id")`). |
| **`elevation.rs`** | Shadow Elevation Mappings | Resolves elevations to shadow arrays. |
| **`floating_menu.rs`** | `FloatingMenuLook` | Styles floating popovers and tooltips. |
| **`focus.rs`** | `FocusRingAdornerSpec` | Styles active ring outlines. |
| **`list_view.rs`** | `ListViewLook` & Rows | Styles tables and flat list content. |
| **`listbox.rs`** | `ListBoxListLook` & Rows | Styles flat select options. |
| **`navigation_sidebar.rs`**| Sidebar Navigation Themes | Styles item lists, triggers, and panel container chrome. |
| **`popup_menu.rs`** | `PopupMenuPalette` | Styles popover menus. |
| **`progress.rs`** | `ProgressLook` | Styles progress bar rails and fills. |
| **`radio.rs`** | `RadioButtonPalette` | Styles choice selector buttons. |
| **`resizable_panels.rs`** | `ResizablePanelsLook` | Styles layout split bars. |
| **`resolve.rs`** | Custom Palette Helpers | Lookup methods with HSL/OKLCH color fallback maps. |
| **`scrollbar.rs`** | `ScrollbarLook` | Styles scroll tracks and handles. |
| **`selection_panel.rs`** | `SelectionPanelLook` | Styles gallery/studio tweak parameter lists. |
| **`selector.rs`** | `SelectorPalette` | Styles select inputs. |
| **`selector_items_panel.rs`**| Selector Dropdown Panel | Styles standard select panels. |
| **`slider.rs`** | `SliderLook` | Styles value slider tracks and thumbs. |
| **`switch.rs`** | `SwitchPalette` | Styles toggle switch tracks and thumbs. |
| **`tabs_navigation.rs`** | Tab Layout Look | Styles horizontal navigation tab bars. |
| **`templates.rs`** | SDK Generic Trait Implementations | Exposes template hooks (`ButtonTemplate`, `SwitchTheme`, etc.). |
| **`textarea.rs`** | `TextAreaPalette` | Styles multi-line text input fields. |
| **`textfield.rs`** | `TextFieldPalette` | Styles standard single-line text inputs. |
| **`tree_view.rs`** | `TreeViewPalette` | Styles hierarchical tree items. |
| **`usage.rs`** | Theme Customization Catalog | Lists widget-level testing states. |

---

### Core Control Implementation Details

Below is the structure of key files linking the parsed tokens to the control-level APIs:

#### [NEW] [controls.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls.rs)
Defines the `ShadcnLookControlExt` builder trait implemented for `Arc<ShadcnLook>`:
```rust
use std::sync::Arc;
use gpui::{AppContext, Entity, SharedString};
use gpui_luma::controls::checkbox::{self, CheckboxBuilder};
use gpui_luma::controls::command::button::{Button, ButtonBuilder};
use gpui_luma::controls::switch::{self, SwitchBuilder};
use crate::look::ShadcnLook;
use crate::button::ShadcnButtonStyle;

pub trait ShadcnLookControlExt {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
}

impl ShadcnLookControlExt for Arc<ShadcnLook> {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Secondary))
    }

    fn primary_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(ShadcnButtonStyle::Primary))
    }

    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        checkbox::new(id).template(self.checkbox_template(ShadcnButtonStyle::Primary))
    }

    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        switch::new(id).template(self.switch_template(ShadcnButtonStyle::Primary))
    }
}
```

---

## Verification Plan

* Ensure `crates/look-shadcn` compiles cleanly with all 31 files registered in `crates/look-shadcn/src/lib.rs`.
* Run `cargo test -p gpui-luma-look-shadcn` to check all tests, including new catalog resolution assertions.

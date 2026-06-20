# Issue #3: Non-Uniform Support for Dynamic Typography Sizing

## Description

Throughout the GPUI-Luma component SDK, the support for dynamic typography scaling based on `ControlSize` (i.e. `sm`, `md`, `lg`) is non-uniform. While controls like buttons, tab bars, and card headers adjust their font sizes when resized, the vast majority of other components (such as text fields, select lists, checkboxes, and popup menus) use static, hardcoded typography values regardless of the container size.

This discrepancy results in an unbalanced user interface when controls are scaled to compact (`sm`) or large (`lg`) dimensions. For example, a `sm` text field adjacent to a `sm` button maintains its default font size (14.0px), creating a noticeable misalignment of text baselines, margins, and visual weight. 

To solve this, we will systematically upgrade the lookless SDK and the Shadcn theme implementation to support uniform font sizing across the codebase, adhering to the user's priority outline.

---

## Code Audit: Non-Uniform Typography Support

### 1. Controls with Dynamic Typography Sizing Support
Currently, only a few components adjust their font size based on the target `ControlSize`:
* **Buttons**: [button.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/button.rs#L126-L129) updates its font size using the rule defined in `style.toml`:
  ```rust
  let mut typography = ctx.typography().text.label;
  if let Some(metrics) = size_metrics {
      typography.size = metrics.font_size;
  }
  ```
* **Tabs Navigation**: [tabs_navigation.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/tabs_navigation.rs#L122-L126) switches its entire text category step:
  ```rust
  let label_typography = match size {
      ControlSize::Sm => typography.text.caption,
      ControlSize::Md => typography.text.label,
      ControlSize::Lg => typography.text.body,
  };
  ```
* **Card Headers**: [card.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/card.rs#L70-L74) scales the card title from `label` to `h4`/`xl`.

### 2. Controls with Hardcoded Typography Sizing
The remaining interactive controls hardcode their font size to a single typography token, ignoring the `ControlSize` parameter:
* **TextField & TextArea**: [textfield.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/textfield.rs#L120) and [textarea.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/textarea.rs) hardcode `typography.text.body` (14.0px).
* **Menus**: [floating_menu.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/floating_menu.rs#L129) and [context_menu.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/context_menu.rs#L41) hardcode `typography.text.label` (12.5px).
* **Selectors**: [selector.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/selector.rs#L34) hardcodes `typography.text.label` (12.5px).
* **Choice Controls**: [checkbox.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/checkbox.rs#L94), [switch.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/switch.rs#L110), and [radio.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/radio.rs#L98) hardcode `typography.text.label` (12.5px).
* **Lists**: [listbox.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/listbox.rs#L135) and [list_view.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/list_view.rs#L149) hardcode `typography.text.label` (12.5px).
* **Accordions & TreeViews**: [accordion.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/accordion.rs#L120) and [tree_view.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/tree_view.rs#L78) hardcode `typography.text.label` (12.5px).

---

## Upgrade Objective & Priorities

The core objective is to upgrade the Luma SDK theme system so that components dynamically map their font-sizes based on the active `ControlSize`. This will be implemented in four sequential phases:

### Phase 1: TextFields & TextAreas (Priority 1)
* **Objective**: Enable full typography scaling for text fields and textareas.
* **Why**: Text fields sit adjacent to buttons on forms, dashboards, and input bars. Unmatched font sizes immediately break baseline grid alignment.
* **Design Pattern**: 
  - `ControlSize::Sm` => `typography.text.label` (12.5px)
  - `ControlSize::Md` => `typography.text.body` (14.0px)
  - `ControlSize::Lg` => `typography.text.body` / `typography.text.body` (14.0px - 16.0px)

### Phase 2: Menus (Priority 2)
* **Objective**: Add `ControlSize` context to Floating Menus, Context Menus, and Popup Menus.
* **Why**: Menus represent primary commands and navigation overlays; when they appear in compact/dense viewports (like sidebar popouts), the text size must shrink alongside padding.
* **Design Pattern**: Pass `ControlSize` through `floating_menu_look` to scale the `item_typography` between `caption`, `label`, and `body`.

### Phase 3: Selectors (Priority 3)
* **Objective**: Upgrade Selector, Autocomplete, Combobox, and Search Selector triggers and popovers.
* **Why**: Selectors mimic buttons and input triggers. The active text selected on the face of the trigger, as well as the list items in the dropdown drawer, must resize together.
* **Design Pattern**: Propagate the trigger size down to the item renderer.

### Phase 4: Remaining Controls (Priority 4)
* **Objective**: Update Checkboxes, Switches, Radio Buttons, Listboxes, List Views, TreeViews, and Accordions.
* **Why**: Complete the SDK's support for responsive typography across secondary controls.

---

## Proposed Implementation Details

To ensure consistency, we will resolve sizes inside [crates/look-shadcn/src/controls/templates.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/templates.rs) and the lookless templates.

1. **Theme Signature Updates**:
   Update `resolve` and `resolve_look` traits in the SDK to accept a `ControlSize` context if they don't already.
   
2. **Typography Role Matching**:
   Use matching `LumaTextStyle` mappings based on control sizes:
   * **Small (`ControlSize::Sm`)**: Map to `typography.text.caption` or `typography.text.label`.
   * **Medium (`ControlSize::Md`)**: Map to `typography.text.label` or `typography.text.body`.
   * **Large (`ControlSize::Lg`)**: Map to `typography.text.body` or `typography.text.title`.

---

## Tasks

### Phase 1: TextField & TextArea Upgrades
- [ ] Add `size: ControlSize` field to `TextField` and `TextArea` models and builders.
- [ ] Update [TextFieldTheme](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/textfield/theme.rs) to accept `ControlSize` in `resolve` or `resolve_look`.
- [ ] Modify [textfield_palette](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/textfield.rs) to map typography size according to `ControlSize`.
- [ ] Verify that Text Field font sizing and icon heights scale uniformly in the gallery.

### Phase 2: Menu Sizing Upgrades
- [ ] Allow passing `ControlSize` to `FloatingMenu`, `PopupMenu`, and `ContextMenu`.
- [ ] Update `floating_menu_look` in [floating_menu.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/floating_menu.rs) to map `item_typography` dynamically (e.g., small uses caption/label, large uses body).
- [ ] Connect trigger size contexts so popups automatically inherits the triggers' sizes.

### Phase 3: Selector & Dropdown Upgrades
- [ ] Add `size` configuration to `Selector`, `Autocomplete`, and `Combobox` controls.
- [ ] Update `SelectorTheme` and `SelectorPalette` to utilize `ControlSize` when returning typography.
- [ ] Scale Selector items list height and typography in response to size configurations.

### Phase 4: Remaining Controls (Checkboxes, Switches, Lists, and Accordions)
- [ ] Add typography scale matching to Checkboxes, Switches, and Radio Buttons.
- [ ] Refactor Listbox rows and List View cells to apply font sizing based on row sizes.
- [ ] Refactor TreeView nodes and Accordion triggers to scale typography.

---

## Acceptance Criteria

- **Form Baseline Consistency**: A `sm` TextField and a `sm` Button placed inline share aligned baselines and matching font scales.
- **Unified Font Scaling**: Standardizing controls to `sm`, `md`, or `lg` will scale their typography context accordingly.
- **Look-Shadcn Conformance**: Theme mappings are loaded from `LumaTypography` tokens rather than using hardcoded pixel constants.
- **Zero Sizing Regressions**: Standard sizing (`md`) preserves original look and metrics.

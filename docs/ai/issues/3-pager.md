# Issue #3: Standalone Paging Control and Customizable Styles

## Description
Currently, the paging UI in GPUI-Luma is represented by `PagingToolbar` and is tightly coupled within the `list_view` control directory (`crates/sdk/src/controls/list_view/toolbar.rs` and `paging.rs`). This couples layout state and formatting directly to list-specific concepts, such as:
- Row selection summaries (`selected_count`, `total_rows`).
- Fixed rendering layouts and hardcoded icon labels.
- Restricted placement logic designed solely for anchoring as a list-view footer.

As a result, the pagination toolbar cannot be reused easily in other paginated views (such as image grids, carousel sliders, document viewers, or search pages). Furthermore, there is no way for a user to customize the style of the pager (e.g., changing from the classic detailed table footer style to a compact prev/next button style or a numeric page-button list).

To solve this, we will extract the pager into a first-class, independent SDK control (`Pager`) with a clean builder API, dynamic metadata slots, and support for multiple pager styles.

---

## Proposed Solution

### 1. Independent Pager Control (`crates/sdk/src/controls/pager`)
We will create a new SDK control module under `crates/sdk/src/controls/pager/` with the standard SDK control structure:

* **[mod.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/pager/mod.rs)**: Exposes public types: `Pager`, `PagerControl`, `PagerEvent`, `PagerBuilder`, `PagerStyle`, `PagerTemplate`, and `PagerTheme`.
* **[model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/pager/model.rs)**:
  Defines `PagerModel` (configuration and state) and `PagerBuilder`.
  ```rust
  pub struct PagerModel {
      pub id: SharedString,
      pub current_page: usize,
      pub page_count: usize,
      pub page_size: usize,
      pub page_size_options: Vec<usize>,
      pub enabled: bool,
      pub style: PagerStyle,
      /// Decouples list selection or custom counts by providing a generic status slot.
      pub info_slot: Option<Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>>,
  }
  ```
* **[control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/pager/control.rs)**:
  Manages runtime state (dropdown menu state for page size) and interaction handlers. Emits events:
  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum PagerEvent {
      PageChanged { page: usize },
      PageSizeChanged { page_size: usize },
  }
  ```
* **[theme.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/pager/theme.rs)**:
  Defines `PagerAppearance` (colors, typography, spacing metrics) and the `PagerTheme` trait.
* **[template.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/pager/template.rs)**:
  Defines the `PagerTemplate` trait and the `ThemedPagerTemplate` factory.

---

### 2. Support for Multiple Pager Styles
We will introduce a `PagerStyle` enum and support multiple layout renderings within the template system:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PagerStyle {
    /// Detailed table pager: displays info_slot summary, "Rows per page" size dropdown,
    /// "Page X of Y", and navigation buttons (<<, <, >, >>).
    #[default]
    Table,
    /// Simple pager: displays "Page X of Y" and compact prev/next arrow buttons.
    Simple,
    /// Minimal pager: displays prev/next buttons only (ideal for tight margins/headers).
    Minimal,
    /// Numeric pager: displays a list of page number buttons with ellipsis markers,
    /// e.g. [<<] [<] [1] [2] [...] [5] [6] [7] [...] [10] [>] [>>].
    Numeric,
}
```

Each style will map to a specific rendering path in the template, using standard button elements, dropdown triggers, and layout alignments to guarantee design consistency.

---

### 3. Decoupling List-View Selection via Info Slot
To avoid coupling the pager directly to lists, the pager will accept a generic `info_slot` closure.
* For the list view, the `PagingListView` wrapper will instantiate the `Pager` and pass the selection summary text to this slot:
  ```rust
  let pager_builder = PagerBuilder::new("list-pager")
      .info_slot(move |_win, _cx| {
          // Inside list view wrapper, resolve selected rows count dynamically.
          div().child(format!("{} of {} row(s) selected", selected, total)).into_any_element()
      });
  ```
* Other applications (e.g. a media viewer or document reader) can use this slot to display "Showing item X - Y of Z" or leave it blank, keeping the pager completely generic.

---

### 4. Refactoring List-View Paging Integration
We will update [paging.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/paging.rs) to consume the new standalone `Pager` control:
* Delete `PagingToolbar` and its template/layout types in [toolbar.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/toolbar.rs) (or delete the file entirely).
* In `PagingListViewControl`, replace `toolbar: Entity<PagingToolbar>` with `pager: Entity<PagerControl>`.
* Map the `PagerEvent` events directly to the list view's paging methods (`set_page`, `set_page_size`).
* Update the declarative macros (`paging_list_view!`) in [macros.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/list_view/macros.rs) to use `PagerBuilder`.

---

### 5. Look Crate & Theming Integration (`gpui-luma-look-shadcn`)
We will integrate pager styling into `crates/look-shadcn` to keep it theme-aware:
* Add `style.toml` rules and default metric values for `pager` borders, margins, padding, size scales, and typography.
* Implement `ShadcnLook::pager_theme()` and `ShadcnLook::pager_template(style)` to resolve themed visuals dynamically.
* Clean up `paging_toolbar_chrome()` from `ShadcnLook` since standard `PagerTheme` and `PagerTemplate` factories will manage it.

---

## Tasks

### Phase 1: Standalone Pager Control Core (`crates/sdk`)
- [ ] Create `crates/sdk/src/controls/pager/mod.rs` to expose the new control.
- [ ] Implement the `PagerBuilder` and `PagerModel` in `model.rs` (supporting `PagerStyle`, `info_slot`, page size options).
- [ ] Implement `PagerControl` in `control.rs` with runtime state, mouse handlers, dropdown logic, and page bounds checks.
- [ ] Implement `PagerTheme` and `PagerAppearance` in `theme.rs`.
- [ ] Implement `PagerTemplate` in `template.rs` with separate rendering paths for `Table`, `Simple`, `Minimal`, and `Numeric` styles.
- [ ] Register the new `pager` module in `crates/sdk/src/controls/mod.rs`.

### Phase 2: Downstream look-shadcn Styling (`crates/look-shadcn`)
- [ ] Register default style rules and palette colors for `pager` elements in `crates/look-shadcn/assets/style.toml`.
- [ ] Implement theme resolution and template mappings for the pager inside `crates/look-shadcn/src/look.rs` and the template factory helpers.
- [ ] Deprecate and remove `paging_toolbar_chrome` from `ShadcnLook` (replacing with `pager_theme`).

### Phase 3: List View Integration Refactor (`crates/sdk`)
- [ ] Modify `crates/sdk/src/controls/list_view/paging.rs` to construct and wire the standalone `PagerControl` instead of the old `PagingToolbar`.
- [ ] Implement the selection summary callback inside list view paging logic and bind it to the pager's `info_slot`.
- [ ] Delete `crates/sdk/src/controls/list_view/toolbar.rs` from the project.
- [ ] Update the `paging_list_view!` macro in `crates/sdk/src/controls/list_view/macros.rs` to align with the new pager builders.

### Phase 4: Application Refactoring & Gallery Showcases
- [ ] Refactor `apps/theme-studio/src/studio/panels/dashboard.rs` and `apps/gallery/src/gallery/panes/list_view/paging_list_view_pane.rs` to construct the list with the new pager builders.
- [ ] Add a new "Pager" showcase pane in `apps/gallery` registry that renders:
  - An interactive preview pane containing the four pager styles (`Table`, `Simple`, `Minimal`, `Numeric`).
  - Controls to configure page size, page count, and state.
- [ ] Verify that theme transitions in Theme Studio correctly update the styling of the new standalone pager.

---

## Verification Plan

### Automated Tests
- [ ] Add unit tests in `crates/sdk/src/controls/pager/control.rs` to test:
  - Transition bounds (clamping current page to `page_count`).
  - Fast-forward pagination step logic.
  - Page-size dropdown events.
  - Numeric page range calculations (generating correctly segmented button lists, e.g. checking ellipses placements).

### Manual Verification
- Run `just gallery` and verify:
  - The new **Pager Showcase** pane renders all pager styles correctly.
  - Page navigation changes and dropdown selections update state dynamically.
  - The **Paging List View** pane functions exactly as before, displaying selection counts correctly in the info slot.
- Run `just theme-studio` and verify:
  - Swapping themes updates colors, border-radii, and text scales for all pager elements.

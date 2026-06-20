# Review Feedback for `0-sdk-review.md`

I reviewed `docs/ai/issues/0-sdk-review.md` against the actual SDK and app structure, without changing code.

## Overall

The direction is good: the document is targeting a real consistency problem, and several of the proposed rules would catch genuine API drift.

But as written, the plan is too broad in some places and too narrow in others. If executed literally, it would produce false positives, miss important public surfaces, and potentially recommend harmful renames.

## Main feedback

### 1. Step 1 inventory is incomplete as written

The document says to list subdirectories under `crates/sdk/src/controls/`.

That is not enough to inventory the real public control surface.

Examples:
- Public file-backed modules also exist in `crates/sdk/src/controls/mod.rs`, not only directories, such as `label`, `menu_item`, and `icon`.
- Some important public controls are nested below families rather than appearing as first-level control directories:
  - `crates/sdk/src/controls/command/button`
  - `crates/sdk/src/controls/command/icon_button`
  - `crates/sdk/src/controls/color/color_arc`
  - `crates/sdk/src/controls/color/color_ring`
  - `crates/sdk/src/controls/color/color_slider`
  - `crates/sdk/src/controls/color/color_field`

**Recommendation:** inventory from `crates/sdk/src/controls/mod.rs` plus nested exported modules, not just first-level subdirectories.

---

### 2. The plan assumes every control has its own `builder + control.rs`, but the SDK has wrapper controls

Several public controls are thin wrappers over shared button machinery.

Examples:
- `crates/sdk/src/controls/checkbox/mod.rs`
- `crates/sdk/src/controls/switch/mod.rs`
- `crates/sdk/src/controls/radio_button/mod.rs`
- `crates/sdk/src/controls/toggle/mod.rs`

These expose checkbox/switch/radio/toggle APIs, but runtime mutation lives in shared button runtime code:
- `crates/sdk/src/controls/command/button/control.rs`

So Step 3's instruction to open each control's `control.rs` is incomplete. For some public controls, the runtime mutators are inherited from a shared underlying type like `Button<bool>`.

**Recommendation:** explicitly audit wrapper layers and their shared runtime implementations.

---

### 3. Rule A is directionally good, but too absolute

The document says `with_` should be reserved strictly for wrapping/transformation helpers, and direct field setters should always be `.x(...)`.

That is a reasonable target, but the current SDK has intentional public APIs that violate that rule and are not obviously accidental.

Examples:
- `crates/sdk/src/controls/command/button/model.rs`
  - `ButtonBuilder::with_data(...)`
- `crates/sdk/src/controls/checkbox/mod.rs`
  - `CheckboxBuilder::with_data(...)`
- `crates/sdk/src/controls/switch/mod.rs`
  - `SwitchBuilder::with_data(...)`

In `ButtonBuilder`, the docs explicitly distinguish `typed(...)` from `with_data(...)`, so this is not random naming drift.

By contrast, there are also `with_` methods that do fit the intended rule well:
- `crates/sdk/src/controls/list_view/model.rs`
  - `with_row_template(...)`

**Recommendation:** either:
- narrow Rule A to “new APIs should follow this convention”, or
- add exceptions for established semantic APIs like `with_data(...)`, or
- explicitly mark these as migration targets and note compatibility impact.

---

### 4. Rule C conflates two different concepts

The document standardizes on:
- builder: `.look_override(...)`
- runtime: `.set_look_override(...)`

That already matches some controls:
- `crates/sdk/src/controls/textfield/model.rs`
- `crates/sdk/src/controls/textfield/control.rs`

But button-family controls currently expose a different concept:
- `crates/sdk/src/controls/command/button/model.rs`
  - `with_look(...)`

That API is not just a post-resolution override; it supplies an look resolver based on full `ButtonRenderModel<D>`.

There is also parity drift today:
- `TextField` has `set_look_override(...)`
- `ListViewBuilder` has `look_override(...)` in `crates/sdk/src/controls/list_view/model.rs`
- but `ListViewControl` does not appear to expose matching runtime `set_look_override(...)`

**Recommendation:** split this rule into two categories:
1. post-resolution look overrides
2. full look resolvers/sources

Otherwise the audit will group unlike APIs together.

---

### 5. Rule D is too broad for the actual control families

The proposed standard is:
- builder: `.value(...)`
- getter: `.value()`
- setter: `.set_value(...)`

That fits several text/numeric controls well:
- `textfield`
- `textarea`
- `slider`
- `scrollbar`
- `progress`

But it does not fit all interactive controls naturally.

Current patterns include:
- `Button::data()` / `Button::set_data(...)` in `crates/sdk/src/controls/command/button/control.rs`
- selection APIs like `selected`, `items`, `query`, etc.

Downstream usage already relies on this distinction. For example:
- `apps/gallery/src/gallery/panes/checkbox/pane.rs`
  - `look.primary_checkbox(...).with_data(false)`
  - `look.secondary_checkbox(...).with_data(true)`

**Recommendation:** limit Rule D to controls whose primary public payload is truly a scalar/text/numeric value. Do not force button-family and selection controls into `value` unless that is an intentional API redesign.

---

### 6. Rule E needs clearer scope, and probably an item-level companion rule

“Every interactive control must support enabled state” is reasonable, but the SDK has multiple enabled layers.

Top-level control examples:
- `AccordionBuilder::enabled(...)`
- `ListViewBuilder::enabled(...)`
- `TextFieldBuilder::enabled(...)`

Item-level examples:
- `AccordionItem::enabled(...)`
- `ControlGroupItem::enabled(...)`
- `ListBoxItem::enabled(...)`
- `NavNode::enabled(...)`
- `MenuItem::enabled(...)`
- `SelectionPanelItem::enabled(...)`

The document should distinguish:
1. top-level control enabled parity
2. per-item enabled parity for composite controls

Otherwise the audit may miss meaningful inconsistencies or flag non-applicable modules.

---

### 7. Step 5 underestimates how apps consume the SDK

The document says to scan `apps/` and `crates/look-shadcn` for builder instantiations like `.new(...)` and entity updates.

That will miss a large part of real downstream usage, because apps mainly consume themed factory helpers rather than raw constructors.

Examples from app usage:
- `apps/gallery/src/gallery/control.rs`
  - `look.split_view(...)`
  - `look.navigation_sidebar(...)`
- `apps/gallery/src/gallery/panes/accordion/pane.rs`
  - `look.textfield(...)`
- `apps/gallery/src/gallery/panes/checkbox/pane.rs`
  - `look.primary_checkbox(...)`

Those themed factories come from:
- `crates/look-shadcn/src/controls/ext.rs`

**Recommendation:** Step 5 should explicitly include:
- `crates/look-shadcn/src/controls/ext.rs`
- themed control factory/helper APIs
- app-side `.update(...)` closures that call runtime setters

A `.new(...)` search alone will undercount usage and may produce bad “orphaned control” conclusions.

---

### 8. The expected audit output format should be defined more tightly

The goal says the deliverable is a markdown document listing all consistency issues. That is fine, but the plan should define the schema for each finding.

Suggested fields per finding:
- control name
- layer (`builder`, `runtime`, `wrapper`, `look-factory`, `app-usage`)
- current API
- expected API / violated rule
- file path
- downstream usages
- migration impact
- compatibility shim needed? (`yes` / `no`)

Without a structured record format, the resulting audit doc may be harder to act on.

## Concrete issues the current plan would likely surface correctly

These look like legitimate findings if the proposed rules stand:

- `SelectionPanelControl::with_template(...)` and `SelectionPanelControl::with_scrollbar_template(...)`
  - `crates/sdk/src/controls/selection_panel/control.rs`
  - These appear to violate the proposed runtime naming rule.

- `ListViewBuilder::look_override(...)` without matching runtime `set_look_override(...)`
  - builder: `crates/sdk/src/controls/list_view/model.rs`
  - runtime parity appears missing in `crates/sdk/src/controls/list_view/control.rs`

## Positive signal

I did not find public builder `set_...` methods in `crates/sdk/src/controls/**/model.rs`, so Rule A’s “no `set_` prefix on builders” is already mostly true in the current codebase.

## Bottom line

`docs/ai/issues/0-sdk-review.md` is a good starting draft, but it should be revised before someone executes it as the formal audit plan.

## Highest-priority doc fixes

1. Inventory from exported public modules, not only subdirectories.
2. Account for wrapper controls built on shared runtimes.
3. Narrow Rule D so `value` only applies where it genuinely fits.
4. Split Rule C’s look override concept from look resolver/source APIs.
5. Expand Step 5 to include `ShadcnLook` factories and wrapper consumption patterns.

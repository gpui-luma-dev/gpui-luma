# SDK Control Lessons From VS Code Shell Customization

## Context

The VS Code shell customization dialog exercised several SDK control boundaries:

- `Button` rows with custom menu-style content.
- `ResizablePanels` used as the workbench side panel host.
- Theme switching while modeless overlays remain open.
- Logical layout regions that can move between physical left/right slots.

The implementation stayed on SDK controls, but exposed some gaps where app code had to know too much about control/theme internals.

## Lessons Learned

### Buttons

- Keep SDK `Button` as the interaction primitive even when the visual row layout is custom.
- Use a local `ButtonTemplate` when the row chrome needs full-width menu behavior, custom spacing, or custom focus/hover shape.
- Custom row content must use the button's resolved foreground, not `chrome.body_text`.
- If a row is selected, hovered, focused, or disabled, theme-resolved button foreground/background may differ from generic chrome tokens.
- Custom glyphs inside buttons should use the same resolved foreground as the row label. Splitting outline/fill between `chrome.border` and `chrome.body_text` caused mismatched and black-looking glyphs.
- Theme changes need explicit notification for child entities owned by overlays. Notifying only the parent app is not enough when the overlay/button entities render independently.

### Resizable Panels

- `ResizablePanels` already supports panel hide/show through `hide_panel`, `show_panel`, `toggle_panel_hidden`, and `is_panel_hidden`.
- `ResizablePanels` does not currently expose panel reordering.
- When a semantic region can move sides, treat panel indexes as physical slots, not logical identity.
- Add mapping helpers at the app boundary, for example `primary_side_bar_panel_index()` and `secondary_side_bar_panel_index()`.
- For primary/secondary side movement, one workable approach is to keep a stable `ResizablePanels` instance and swap the render content assigned to the physical left/right edge slots.
- Activity bars and sidebars are logical pairs. Moving the primary side must move both the primary activity bar and primary side bar; the secondary activity bar and secondary side bar move to the opposite side.
- Explicitly setting the `ResizablePanels` frame size from the containing viewport avoided delayed layout updates that otherwise resolved only after mouse movement.

### Layout

- SDK `GridLayout` was useful for row composition, but fixed columns can easily overflow small dialogs.
- For menu rows, prefer fixed utility columns plus a `Star(1.0)` label/content column.
- Right-side hint labels should live inside the same full-width button row layout when they are visually associated with that row group. Placing them outside the row can make hover/selection look clipped or misaligned.

## Recommended SDK Enhancements

### Button Content Presenter

`Button` should support a lighter-weight content presenter API that receives resolved theme/state context. This would let app code customize content without calling look-specific theme resolvers directly.

Example target API:

```rust
.content_presenter(move |ctx| {
    hstack! {
        layout_glyph(ctx.foreground),
        div()
            .text_color(ctx.foreground)
            .child("Primary Side Bar"),
    }
})
```

The context should include both resolved tokens and raw state:

```rust
pub struct ButtonContentContext<'a, D> {
    pub id: &'a SharedString,
    pub data: &'a D,
    pub state: InteractionState,
    pub role: ControlRole,
    pub size: ControlSize,
    pub foreground: Hsla,
    pub background: Hsla,
    pub typography: LumaTextStyle,
}
```

State is needed for conditional content such as selected checkmarks, disabled hints, alternate glyphs, badges, or counters. Resolved visual tokens are needed so app-specific content follows theme hover/selected/focus behavior.

### Selectable Menu Row / Radio Item Layout

The primary side bar position controls were visually menu-choice rows, but behaviorally radio options. A better SDK shape would be a radio/control-group item layout that can render as full-width menu rows.

Example target API:

```rust
let side_position = look
    .radio_group("primary-side-bar-position")
    .value(config.primary_side_bar_position)
    .items([
        PrimarySideBarPosition::Left,
        PrimarySideBarPosition::Right,
    ])
    .with_item_layout(move |item, ctx| {
        grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(POSITION_LABEL_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];

            [0, 0] => layout_option_icon(item.value, ctx.foreground),
            [0, 1] => hstack! {
                gap=8 align=center;
                div()
                    .typography_style(ctx.typography)
                    .text_color(ctx.foreground)
                    .child(item.value.label()),
                selected_check(ctx.selected, ctx.foreground),
            },
            [0, 2] => position_header(item.index == 0, ctx.muted_foreground),
            [0, 3] => div().w(px(EYE_COL_W)),
        }
    })
    .row_template(MenuRowLikeTemplate)
    .spawn(cx);
```

The SDK group would own "only one item selected" semantics. The app would subscribe once to a change event rather than maintaining two independent buttons and manually syncing selected state.

Useful item context:

```rust
pub struct RadioItemLayoutContext {
    pub selected: bool,
    pub disabled: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
    pub background: Hsla,
    pub typography: LumaTextStyle,
}
```

This can still render like a VS Code menu row, not a circular radio control. The radio/control-group layer provides behavior; the item layout provides presentation.

### Resizable Panel Logical Regions

`ResizablePanels` might benefit from an optional logical-region layer for app shells:

- Stable region IDs separate from physical panel indexes.
- Hide/show APIs by region ID.
- Optional move/reorder APIs when a region changes edge.
- Event payloads that include both physical index and logical region ID.

This would reduce app-side mapping code for workbench-like shells where "primary" and "secondary" can swap physical sides.

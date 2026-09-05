# Look / SDK TODO

## SDK variant cleanup

- Inventory every public SDK `Variant`, `Style`, and recipe-like enum.
- Classify each as behavioral, structural, or visual-only.
- Remove visual-only variants from SDK control models.
- Move visual recipes into look-owned factories and templates.
- Keep structural variants when they change layout or interaction behavior.
- Split mixed enums where one type currently combines state, layout, and visual recipe concerns.

Initial candidates:

- Remove `TextFieldVariant::Standard` if no second textfield behavior is needed.
- Move popup-menu trigger styles to look-owned factories/templates.
- Move selector trigger styles to look-owned factories/templates.
- Move toolbar visual variants to look-owned factories/templates.
- Review `ButtonFamilyRole` for separation of icon layout, toggle state, and visual styling.
- Review `PagerStyle` and `ScrollbarStyle` for visual-only cases.

## Template boundary

- Keep custom templates as a primary SDK capability.
- Keep template modifiers for small presentation changes.

## `look-core` cleanup

- Keep shared resolved-value and provenance types in `look-core`.
- Remove duplicate equivalent provenance types from `look-shadcn`.
- Do not move Radix/Shadcn visual recipes or token names into `look-core`.
- Add a shared contract only when both looks independently need it.
- Revisit shared theme revision/invalidation after `look-radix` uses the SDK invalidation path.

## Completion check

- A look can define its own templates and visual recipes without SDK changes.
- SDK variants describe behavior or structure, not design-system vocabulary.
- `look-core` contains only genuinely shared contracts.

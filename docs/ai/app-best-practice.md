# App Best Practice

This note describes how code under `apps/*` should be written in this repository.

The short version:

- Apps compose the SDK.
- Apps do not reinvent the SDK.
- Prototypes should separate the demo pane from the thing being prototyped.
- Prefer existing layout/helpers over hand-built scaffolding.
- Keep code explicit, reviewable, and free of avoidable magic values.

## 1. Respect the SDK/App Boundary

The SDK in `crates/sdk` is the source of truth for interactive controls and shared UI behavior.

Apps under `apps/*` should:

- use SDK controls and builders
- use `ShadcnLook` factories and extensions where appropriate
- wire subscriptions, routing, app state, and page composition
- provide static copy and shell layout

Apps under `apps/*` should not:

- invent alternate buttons, checkboxes, toggles, inputs, menus, tabs, or other interactive chrome with raw `div` styling
- reimplement focus, hover, pressed, disabled, or selection behavior already handled by SDK controls
- duplicate theme-token-driven visuals locally when the SDK already owns that control family

If something is missing, the default answer is:

1. extend the SDK or the relevant look/template seam
2. then consume that seam from the app

Do not patch around missing SDK capability with one-off app chrome unless it is truly static, non-interactive layout.

## 2. Use Existing Layout Helpers

If the code is expressing a stack, row, or wrap layout, prefer the existing helpers from `gpui_luma::macros`:

- `vstack!`
- `hstack!`
- `flow!`
- `wrappanel!`
- `declare_form!` and `form_field!` when a form-shaped pattern fits

These helpers are not cosmetic. They:

- reduce repetitive flex boilerplate
- make intent obvious in reviews
- keep app code aligned with the rest of the repo

Do not hand-write long chains of `.flex().flex_col().gap(...)` or `.flex().items_center().gap(...)` when a macro expresses the same layout more clearly.

Also reuse existing shared pane helpers when they already fit:

- gallery pane shells
- inspector shells
- section/header helpers
- shared notify helpers

Prefer consistency over local cleverness.

## 3. Prototype Structure

When building a prototype in the gallery:

- keep the prototype visible and runnable in `apps/gallery`
- keep the demo pane separate from the prototype artifact itself
- do not bury reusable behavior inside the pane if it is the thing being evaluated

Recommended shape:

- `pane.rs` owns the gallery/demo shell, explanatory copy, trigger wiring, and scenario setup
- a sibling module or folder owns the actual prototype behavior/rendering/state being evaluated

For example:

- `pane.rs` = demo surface
- `control/`, `button.rs`, `template.rs`, or similar = prototype artifact

This separation matters because:

- the pane is disposable demo code
- the prototype artifact is the part most likely to graduate into a reusable seam
- mixing them makes review, migration, and cleanup harder

If a prototype is specifically testing whether something belongs in the SDK, structure it so the prototype artifact can be moved with minimal untangling.

## 4. Prefer Existing Seams Before Inventing New Ones

Before creating a new abstraction:

- inspect the relevant SDK control, template, and theme modules
- check whether a template hook, builder option, theme resolver, or look extension already exists
- compare with nearby controls that solve a similar problem

Typical good seams:

- control template replacement
- look/theme resolver extensions
- builder configuration
- shared control state/model updates

Typical bad seams:

- wrapper abstractions that only exist to avoid touching the real control
- app-local reimplementations of shared control visuals
- one-off styling branches in app code that should live in a control/template/theme layer

Prefer the real seam over a workaround seam.

## 5. Avoid Magic Numbers and Strings

Do not scatter raw measurements, opacities, IDs, labels, or status literals through render code when they represent stable design intent.

Prefer:

- named constants for spacing, sizing, typography, opacity, and durations
- helper functions for repeated copy or repeated render fragments
- enums instead of ad hoc string state where possible
- stable IDs with deliberate naming

Raw literals are fine when they are genuinely one-off and obvious. They are not fine when the reader has to infer whether `18.0`, `22.0`, and `24.0` are meaningful design tokens or random drift.

Good code should answer:

- what is this value for?
- is this value reused elsewhere?
- should this value change together with related values?

If yes, name it.

## 6. Keep Render Code Readable

Render functions should read like structure, not like a wall of chained styling calls.

Prefer:

- small helper render functions for obvious subsections
- layout macros for primary structure
- grouping by intent: header, stage, controls, status, footer

Avoid:

- giant render functions with deeply nested anonymous `div()` chains
- mixing event logic, layout, and copy updates in one long block
- repeating the same render fragment with minor changes inline

Good render code should make the UI hierarchy easy to scan.

## 7. Keep App Logic Thin

App code should primarily do:

- state orchestration
- subscriptions
- routing
- demo setup
- composition of controls

Push shared behavior downward when it is not app-specific.

Examples of logic that often belongs below the pane level:

- reusable focus handling
- overlay positioning
- animation state
- control-local interaction state

Examples of logic that often belongs in the pane:

- which demo buttons open which scenario
- what explanatory text the gallery shows
- what status message a prototype displays

## 8. Reuse Repo Patterns

When touching existing app code:

- preserve the established visual and structural patterns
- follow nearby files before introducing a new style
- prefer the same naming scheme used by adjacent modules

If a neighboring pane already has a good pattern for:

- headers
- sections
- subscriptions
- notify helpers
- inspector composition

reuse that pattern instead of inventing a parallel one.

## 9. Keep Diffs Small and Reviewable

Prefer small, direct changes over wide refactors.

When possible:

- make one structural improvement at a time
- separate behavior changes from cleanup
- avoid opportunistic rewrites of unrelated code

A good diff should make it easy to answer:

- what changed?
- why did it change?
- what behavior should I verify?

## 10. Verify the Result

At minimum, validate the target you changed.

Common expectations:

- run `cargo fmt`
- run `cargo check` for the relevant crate
- for gallery work, usually `cargo check -p gpui-luma-gallery`
- manually verify the affected pane or interaction path when the change is visual or behavioral

If you could not run verification, say so explicitly.

## 11. Prefer Explicitness Over Cleverness

This repo benefits more from clarity than compression.

Prefer:

- explicit helper names
- obvious control flow
- straightforward data ownership
- small local abstractions with clear purpose

Avoid:

- clever generic layers that hide intent
- dense indirection for simple UI work
- unnecessary macro or trait complexity in app code

The goal is maintainable, migratable code, not novelty.

## 12. Practical Checklist

Before finalizing app-layer work, check:

- Am I using an existing SDK control instead of inventing one?
- Did I use the repo’s layout helpers where they fit?
- If this is a prototype, is the pane separate from the prototype artifact?
- Did I inspect the real SDK/control seam before adding a workaround?
- Are repeated numbers and strings named or otherwise justified?
- Is the render tree readable without mentally parsing a styling wall?
- Did I reuse existing repo patterns where possible?
- Did I run formatting and relevant verification?

If the answer to several of these is no, the code probably needs another pass.

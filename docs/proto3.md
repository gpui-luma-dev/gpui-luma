# Proto3: Status Summary & Next Decisions (Prototype-to-Refactor Track)

## Status

- **Type:** Summary + decision document
- **Predecessor:** `docs/proto2.md`
- **Scope:** Prototype paths remain isolated (`controls/prototypes/*`, gallery prototype pane)

---

## Why this doc exists

Proto2 established the core technical direction for state-aware template parameterization.  
This document captures:

1. what is currently true in the prototype
2. what is still unfinished
3. what decisions need to be made next for broad control refactoring

---

## Current Snapshot (as of Proto3 kickoff)

### Completed in prototype architecture

1. **State-aware overrides are implemented**
   - `ProtoButtonStatefulOverride<T>` supports:
     - `base`
     - `hovered`
     - `pressed`
     - `focused`
     - `disabled`
   - `ProtoButtonVisualState` is used for deterministic state targeting.

2. **Nullable override semantics are explicit**
   - `ProtoButtonNullableOverride<T>` supports:
     - `Inherit`
     - `Set(T)`
     - `Clear`
   - Focus-ring behavior is deterministic under these semantics.

3. **Deterministic resolution is centralized and test-backed**
   - Resolver precedence is explicit:
     1. state-specific override
     2. base override
     3. defaults/theme fallback
   - Nullable semantics are handled in the same deterministic pipeline.
   - Unit tests cover precedence + nullable behavior.

4. **Prototype moved toward a clearer greenfield pipeline**
   - `defaults.rs` (baseline defaults model + defaults source abstraction)
   - `resolver.rs` (pure resolution logic)
   - `resolved_style.rs` (final resolved style + source metadata)
   - template render path consumes resolved style.

5. **Gallery prototype demo behavior advanced**
   - state preview is present
   - selected-state editing loop is present
   - effective/source inspector behavior exists
   - wording in prototypes moved from “emergency” to “demo”
   - destructive usage was removed from prototype gallery flow.

---

## What remains incomplete from Proto2 goals

1. **Metadata v2 is still string-heavy**
   - Usage metadata still relies on manual string field paths.
   - This is a known maintainability and scaling risk.

2. **Two-way panel (Step 5) is optional and only partially matured**
   - Working loop exists, but not fully generalized as schema-driven UI.

3. **Docs closeout is incomplete**
   - Proto1 cross-references and final migration constraints need final pass.
   - Proto2 reflects implementation progress but not full finalization language.

---

## Key Insight from implementation + UX iteration

The biggest user-facing confusion surfaced around **effective source clarity**:
- users change one visual field and expect full visual intent to follow
- unresolved/implicit fallback (theme/base/state) can feel surprising

### Conclusion
For a production-grade refactor path, **explainability is first-class**:
- every editable field should make source explicit (`theme`, `base`, `state`, `set`, `clear`)
- panel behavior should be driven by resolver truth, not duplicated heuristics.

---

## Proto3 Decisions Needed

## 1) Metadata direction: keep manual strings vs generated typed metadata
**Decision target:** Move toward generated metadata (derive/macro) with typed field/state identifiers and late string rendering only at UI boundary.

Rationale:
- avoids drift
- compile-time safety
- scalable across all controls

### Implementation note (selected direction)
Adopt a derive-driven metadata plan for prototype params, e.g. `#[derive(ProtoComponent)]` with field-level `#[proto(...)]` attributes for defaults/state applicability and docs-derived descriptions. The intended effect is to remove hand-authored string field paths (for example, `"ProtoButtonTemplateParams.variant"`) from usage registration and generate that shape consistently.

The actual struct definition would look like (using current variant naming, where `Standard` is the neutral default, `Prominent` is the loud/solid action style, and `Ghost` is the quiet/transparent style):
```rust
#[derive(ProtoComponent)]
#[proto(prefix = "ProtoButton")] // Automates "ProtoButtonTemplateParams.variant"
pub struct ProtoButtonTemplateParams {
    /// Base variant resolved from theme before applying parameter overrides.
    #[proto(default = "ButtonVariant::Standard", states = "standard")]
    pub variant: ButtonVariant,

    /// Base size resolved from theme.
    #[proto(default = "ControlSize::Md", states = "standard")]
    pub size: ControlSize,
    
    /// Opacity applied when the control is disabled.
    #[proto(default = "0.56", states = "disabled")]
    pub disabled_opacity: f32,
    
    /// Whether to set pointer cursor when enabled.
    #[proto(default = "true", states = "standard")]
    pub pointer_cursor_when_enabled: bool,
    
    /// Background override with base + per-state values applied after theme resolution.
    #[proto(states = "standard")]
    pub background: ProtoButtonStatefulOverride<Hsla>,
    
    /// Foreground override with base + per-state values applied after theme resolution.
    #[proto(states = "standard")]
    pub foreground: ProtoButtonStatefulOverride<Hsla>,
    
    /// Border color override with base + per-state values applied after theme resolution.
    #[proto(states = "standard")]
    pub border: ProtoButtonStatefulOverride<Hsla>,
    
    /// Focus ring override semantics: Inherit uses theme, Set(color) forces a color, Clear removes the ring.
    #[proto(states = "focused")]
    pub focus_ring: ProtoButtonNullableOverride<Hsla>,
    
    /// Corner radius override.
    #[proto(states = "standard")]
    pub radius: Option<f32>,
    
    /// Horizontal padding override.
    #[proto(states = "standard")]
    pub padding_x: Option<f32>,
    
    /// Vertical padding override.
    #[proto(states = "standard")]
    pub padding_y: Option<f32>,
    
    /// Content gap override.
    #[proto(states = "standard")]
    pub gap: Option<f32>,
    
    /// Control height override.
    #[proto(states = "standard")]
    pub height: Option<f32>,
    
    /// Typography size override.
    #[proto(states = "standard")]
    pub typography_size: Option<f32>,
    
    /// Typography line height override.
    #[proto(states = "standard")]
    pub typography_line_height: Option<f32>,
    
    /// Typography weight override.
    #[proto(states = "standard")]
    pub typography_weight: Option<FontWeight>,
}
```

When implemented, this approach would generate the metadata automatically, eliminating the need for manually maintained string paths in `PROTO_BUTTON_TEMPLATE_USAGE`. The derive macro would:
- Automatically generate field identifiers for all parameters
- Create typed `ProtoButtonTemplateParamField` enum variants
- Handle state-specific field mapping (base, hovered, pressed, etc.)
- Generate documentation from field comments
- Provide compile-time safety against field name drift

This plan requires introducing a proc-macro parsing stack and using `darling` for attribute parsing/validation in the derive crate.

---

## 2) Semantic role model vs distributed variant branching
**Decision target:** Treat semantic style choices as centralized presets/roles, not distributed template branching.

Rationale:
- adding a new role should not require touching many files
- resolver should ingest role/preset data once, then apply standard precedence.

---

## 3) End-user experience contract
**Decision target:** Define a strict UX contract:
- raw value (if any)
- effective value
- source provenance
- selected state context

Rationale:
- eliminates ambiguity during runtime editing
- matches prototype's purpose as future architecture exemplar.

---

## 4) Focus styling track alignment
Focus/adorner exploration remains a parallel concern (`docs/adorner.md`) and should stay decoupled from Proto3 metadata/resolver decisions except where resolver semantics require nullable handling and source clarity.

---

## Proposed Proto3 Workstreams

## A) Metadata correctness + generation
- Introduce typed metadata shape for prototype params.
- Add a derive-based metadata generator (`ProtoComponent`) for prototype parameter structs.
- Parse and validate derive attributes with `darling`.
- Remove stringly field-path maintenance from hand-authored tables.
- Add completeness tests for metadata coverage and derive output stability.

## B) Resolver-as-source-of-truth UI
- Make gallery inspector/panel read directly from resolved-style provenance.
- Ensure displayed source matches actual resolver output.

## C) Role/preset centralization prototype
- Prototype a role/preset layer in `prototypes` only.
- Keep compatibility with existing defaults source while avoiding distributed per-variant styling logic.

## D) Documentation hardening
- Finalize Proto2 outcome notes.
- Add explicit migration notes for incremental rollout to non-prototype controls.
- Document the new derive-metadata contract and dependency implications (including `darling`) for contributors.

---

## Proto3 Exit Criteria

Proto3 can be considered successful when:

1. Metadata is no longer fragile/stringly for prototype params.
2. Resolver provenance is the canonical source used by tooling/panel UI.
3. Adding a new semantic style role does not require broad template edits.
4. Prototype docs clearly define migration path and constraints for core adoption.
5. Prototype remains isolated and non-disruptive to existing controls.

---

## Summary

Proto2 proved that state-aware, deterministic template parameterization works in practice.  
Proto3 is about making that foundation **scalable, explainable, and maintainable** enough to serve as the model for wider control refactoring.
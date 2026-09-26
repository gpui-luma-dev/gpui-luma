# TreeView #77 implementation plan

Review baseline: `8941ad3f`, 2026-09-26. Implementation progress is recorded below.

Issues: [#77](https://github.com/scottcg/gpui-luma/issues/77),
[#44](https://github.com/scottcg/gpui-luma/issues/44),
[#76](https://github.com/scottcg/gpui-luma/issues/76).

## Recommendation

Keep the existing entity control and `ListState` renderer. Deliver small changes around a concrete workflow: moving project folders and documents between two in-memory workspaces. Nodes have stable IDs independent of their paths, a title, kind, icon, metadata, and an optional SDK action button. Empty folders, disabled destinations, and expanded nested branches make targeting observable. No filesystem operations.

Use Luma Studio → Controls → TreeView as the test area for all examples and interaction testing, including the two-workspace DnD fixture, filtering, positioning, rich templates, and large-tree diagnostics. Extend `apps/luma-studio/src/studio/controls/control_exposition/tree_view.rs`, splitting fixtures into adjacent modules as needed. Keep sample mutation and acceptance rules in Studio. This user-selected location supersedes #44's original Developer-tab placement; coordinate #44's DnD acceptance criteria with #77 in this same exposition.

## Implementation progress

Slice 1 is implemented:

- SDK and Shadcn `.try_items(...)` validate all loaded IDs. The existing `.items(...)` builder retains its previous data on invalid input.
- `replace_items(items, cx)` preserves eligible selection, expansion, active identity/fallback, and a surviving top-row key/offset. `try_set_items(...)` provides a fallible reset; legacy `set_items(...)` ignores invalid replacements. Replacement applies initial expansion hints only to new node IDs and does not synthesize expansion events.
- Replacement invalidates row measurements even when counts match. Retained row callbacks resolve stable IDs. Selection/active transactions emit changed final state only; repeated selection and no-selection mode no longer produce spurious selection events.
- Controls → TreeView includes an independent state-update fixture with SDK buttons for reorder, reparenting Guide, removing selected nodes, duplicate-ID rejection, and reset. Its headless test clicks the actual SDK buttons. The event tracker now includes existing scroll and hover events.

Slice 2 is implemented:

- The optional SDK scrollbar (included by Shadcn) uses the existing `ListState` for position, viewport, measured extent, and drag lifecycle. Both Studio scroll shells have been removed. Theme refresh updates both tree and scrollbar.
- `require_focus_for_scroll` preserves hover scrolling by default and passes unfocused/disabled wheel input to the enclosing page when enabled. Accepted wheel input remains contained at endpoints. The adapter snapshots/restores native list movement during dispatch; it requires no GPUI fork.
- A dedicated TreeView key context binds Left/Right. Child/ancestor traversal skips ineligible rows; empty branches never navigate to siblings. `wrap_navigation(false)` enables bounded navigation. Offscreen keyboard reveal handles unmeasured rows.
- Hover and press state use node IDs. Collapse immediately reconciles active state and disables closing descendants while preserving hidden selection. Completing several collapses preserves a surviving keyed scroll anchor. Turning animation off settles pending expansion/collapse.
- Controls → TreeView starts with enough expanded rows to scroll and enables focus-required scrolling and bounded navigation. Headless dispatch tests cover page routing, endpoints, focus exit, actual scrollbar dragging, resizing, data shrinkage, keyboard reveal, and collapse completion.

Slice 3 is implemented:

- SDK/Shadcn `branch_content` and `leaf_content` accept owned UI callbacks, including non-Send captures, inside the themed row shell. Named and inline examples share a local construction macro in the Studio workspace fixture. Full templates remain available; legacy full templates must implement `render_node_with_content` to support content callbacks.
- `expand_on_row_click(false)` makes pointer disclosure independent of row selection. Embedded SDK Info buttons use the shared drag boundary. The control supplies scoped row IDs and accessible tree/treeitem roles, levels, selection and expansion state.
- `TreeViewDragDrop` cooperates by scope and domain type. Native single-node captures use the shared keyed session/lifecycle helpers. `TreeDropProposal` contains the target parent, sibling anchor, and Before/After/Into intent; After inserts after the anchor's entire subtree. Blank viewport space appends roots, including empty destinations.
- Validation covers source/target availability, source data revisions, retained target revisions, enabled nodes/parents, anchor parent identity, cycles, subtree-wide ID collisions, host acceptance and ended/foreign sessions. Host mutation remains synchronous in Studio and prepares/validates both snapshots before applying either. Same-tree moves retain keyed state; cross-tree moves explicitly capture/restore subtree state. Notifications distinguish moves within a tree from transfers and complete once.
- Themed markers/preview, Escape and outside-release cancellation, and unfocused destination edge auto-scroll are integrated with the existing list viewport. The linear edge response is shared with ListBox. Auto-scroll revalidates targets every frame, including under a stationary pointer, and stops on invalidation/cancellation.
- Controls → TreeView has the two-workspace fixture, with folders, empty/disabled destinations, metadata, independent SDK actions and a lifecycle event log.

Slice 4, filtering and sorting, is implemented:

- SDK/Shadcn builders accept `filter` and stable sibling `sort` callbacks; live controls provide setters and clear methods. A cached display hierarchy stores source indices, leaving domain data and source sibling order intact. Callbacks run when configured or when data is replaced, rather than during rendering.
- Filtering evaluates loaded nodes and includes matches plus ancestor paths, without automatically including unmatched descendants of matching branches. Required ancestors stay open independently of saved user expansion; expansion commands on those ancestors leave saved choices unchanged. Clearing restores user expansion. Hidden selections survive, active identity falls back in displayed order, and retained scroll anchors remain keyed.
- Row construction, motion updates, keyboard navigation, and transferred-state reconciliation use the same projection. Projection changes settle old motion and invalidate retained drag proposals. Filtered/sorted destinations reject Before/After while retaining keyed Into and root append.
- Controls → TreeView's two-workspace example now includes SDK name/detail search and sibling-order selection, plus visible-row and hidden-selection counts. Its event stream is enclosed in the standard log, workspace headings use theme colors, and the exposition code sample is removed.
- Headless tests cover expansion restoration, collapsed matches, branch-only matches, no results, non-Send callback captures, stable sorting, keyboard order, replacement/event reconciliation, scroll anchoring, animation interruption, stale proposals, actual projected drops, and Studio control subscriptions.

Filtering/sorting verification passed: all 464 SDK library tests (34 TreeView tests), two Shadcn TreeView tests, eleven Studio TreeView tests, formatting, and workspace/all-target Clippy with warnings denied. The user has also tested filtering/sorting successfully.

Slice 4, positioning and smooth motion, is implemented:

- `scroll_to` minimally reveals and `scroll_to_center` centers a displayed stable ID; both have `_smooth` counterparts. Requests do not change selection, active identity, or focus. Missing/collapsed/filtered IDs are ignored. `reveal_node` and `reveal_node_smooth` explicitly open loaded ancestors without fetching or bypassing filters.
- A pending keyed request uses the existing ListState; final alignment uses measured row/viewport bounds and native clamping. Unmeasured jumps use logical row estimates until actual bounds are available. Oversized rows reveal their leading edge. Layout can finish measurement corrections after the shared 200 ms cubic easing interval.
- Accepted wheel input, pointer presses, navigation, scrollbar interaction, native dragging, new requests, and data/projection/expansion changes cancel positioning. Smooth requests also stop when the window is inactive. Pending requests wait for nonzero layout and existing disclosure motion. Delayed frame notifications cannot revive a cancelled request.
- Controls → TreeView includes a compact Positioning fixture with target/movement selectors, Go, Close & show folder, and the standard event log. The close action brings Example folder into view and reports closed/already-closed status. SDK tests cover measured alignment, variable and oversized row heights, boundaries, no-op positioning, hidden/missing keys, ancestor reveal, sorting/filtering, intermediate frames, replacement/interruption, and deferred layout. A Studio test clicks Go for centering and hidden-descendant reveal.

Positioning verification passed: all 470 SDK library tests (40 TreeView tests), two Shadcn TreeView tests, twelve Studio TreeView tests, formatting, and workspace/all-target Clippy with warnings denied. No GUI was launched.

The user has tested positioning and the Close & show folder follow-up successfully.

### Extended selection and ranges

- Added opt-in Extended selection with plain-click replacement, Ctrl/Cmd toggles, Shift ranges and additive Ctrl/Cmd+Shift ranges. Ranges follow effective displayed preorder, skip disabled/closing rows, and never wrap. Active and anchor IDs reconcile through collapse, projection, replacement, and subtree transfer.
- Extended Up/Down/Home/End navigate without selection; Shift selects ranges, Space selects/toggles, and Enter emits `NodeActivated`. Ctrl/Cmd+A adds visible eligible nodes while preserving hidden selections; Ctrl/Cmd+Shift+A explicitly clears all. Existing Single/Multiple keyboard defaults remain unchanged.
- Added runtime selection policy, optional plain-click deselection, Single-only selection following, select-all/clear APIs and actions, and deduplicated selection/policy events.
- Controls → TreeView → Selection and ranges uses SDK mode/sort selectors, policy checkboxes, search, Select visible/Clear buttons, selected/hidden counts, active/anchor status, and the enclosed event log.
- Headless tests cover real modifier clicks and keyboard dispatch, range contraction, projections, disabled/hidden nodes, anchor reconciliation, runtime policies, drag selection preservation, and embedded SDK input boundaries.

Verification passed: all 477 SDK library tests (47 TreeView tests), two Shadcn TreeView tests, fifteen Studio TreeView tests, formatting, and workspace/all-target Clippy with warnings denied. Studio/workspace commands used `--features test-support,gpui_platform/runtime_shaders`. No GUI was launched.

Selection visibility follow-up: the user reported multi-selection appeared broken, then confirmed the selected count increased. Both default SDK and Shadcn row themes ignored the selected flag. Selected rows now retain the theme's selection background and contrasting label/icon/chevron colors independently of hover or focus. Shadcn also resolves interaction colors using the actual light/dark mode. A regression test failed before the fix; 48 SDK, three Shadcn, and fifteen Studio TreeView tests now pass, along with formatting and workspace/all-target Clippy. Manual retest remains with the user; no GUI was launched.

The user has retested Extended selection successfully after the selection-highlight fix.

### Grouped moves

- Added opt-in `TreeViewDragDrop::drag_selected(true)`. Dragging a selected row captures visible eligible selected roots in displayed preorder, omitting selected descendants of selected ancestors. Entire subtrees travel with their root; unrelated hidden selections stay in the source. Unselected rows still drag alone.
- Proposals expose all roots through `node_ids()`; lifecycle events report the same group once. The preview reports the root count. `node_id` remains the first moving root. Validation checks every subtree for cycles and collisions and retains scope/stale-session guards.
- Added `capture_subtrees_state` to transfer group selection, expansion, active identity, and range anchor together. Studio prepares both snapshots before applying either, extracts roots in one traversal, and resolves sibling gaps before removal so moving anchors and unchanged drops work consistently.
- Working / Archive now uses Extended selection and grouped dragging. Tests cover actual grouped pointer drags, ordered roots, normalization, collapsed/filtered selections, sorted sources, atomic observer state, destination selection retention, same-tree reparenting/no-ops, second-root collisions, cycles, stale proposals, rejection, cancellation, and one completion.

Grouped-move verification passed: all 480 SDK library tests (50 TreeView tests), three Shadcn TreeView tests, twenty-one Studio TreeView tests, formatting, diff checks, and workspace/all-target Clippy with warnings denied. No GUI was launched.

The user has tested grouped moves successfully.

### Large-scale validation

- Added Controls → TreeView → Large tree — 10,117 nodes, at the bottom of the exposition. It contains 100 folders × 100 documents plus a 16-level branch. SDK controls provide first/middle/deep jumps, same-count sibling reversal, folder-50 collapse/expand, filtering, reset, and sampled construction counters. Loaded/displayed counts, unique row IDs constructed, and total row-template calls are reported separately. Counters accumulate until explicitly reset; sampling avoids a continuous diagnostic redraw loop.
- Added SDK TestWindow cases with 10,133 loaded/displayed nodes and a 32-level branch, plus a single branch containing 10,000 children. Tests cover distant positioning, content replacement and reordering with unchanged counts, keyed scroll/selection preservation, filtering/sorting, deep reveal, and animation completion while scrolled.
- Scale testing exposed row-construction growth as animated descendant heights shrink. Simultaneous transitions affecting over 256 displayed descendants now settle before layout; smaller transitions keep their existing animation. A shared budget covers concurrent branches. This bounds animation-related construction without claiming constant-time projection or data cloning.
- Headless measurements with a 240px viewport: initial display constructed 24 unique rows / 40 template calls; distant jump 52 / 70; same-count reorder 41 / 41; collapse above viewport 41 / 59; deep filter 24 / 32; deep reveal 49 / 57. The 10,000-child branch settled collapse with 2 / 4 and expansion with 24 / 32. Each operation passed bounds of fewer than 200 distinct constructed rows and 1,000 template calls. These are TestWindow construction counts, not native renderer timings or frame-rate claims.
- Verification passed: all 484 SDK library tests (54 TreeView tests), three Shadcn TreeView tests, twenty-two Studio TreeView tests, formatting, diff checks, and workspace/all-target Clippy with warnings denied. No GUI was launched.

The user has tested the large-tree example successfully. Follow-up: exposition actions now use SDK enabled state. The large-tree filter disables hidden jumps, folder-50 toggling, and repeat filtering; Clear filter and Middle track filter/expansion state. Selection actions and policy checkboxes follow mode and eligible membership. State-update Move/Remove/Reorder require applicable data, Positioning Go requires a visible target or an ancestor-reveal movement, and the current scrollbar policy button is disabled. Close & show folder stays available because it also scrolls to the folder. Reset and diagnostic/rejection actions remain available. All 22 Studio TreeView tests pass, including disabled-state and disabled-click assertions, along with formatting, diff checks, and workspace/all-target Clippy. No GUI was launched.

All planned implementation slices are complete, and the user has confirmed the enabled-state follow-up works. Current rich content uses theme-defined row heights. Auto-scroll is linear; hover-to-expand remains deferred. The local macro stays in Studio. Scrollbar extent follows GPUI’s measured-row geometry and can grow as additional rows are measured.

Slice 3 verification passed: 458 SDK library tests (including shared drag/drop and 28 TreeView tests), two Shadcn TreeView tests, eight Studio TreeView tests, 97 ListBox regression tests across SDK/look/Studio, formatting, and workspace/all-target Clippy with warnings denied. Studio/workspace commands used `--features test-support,gpui_platform/runtime_shaders`. No GUI was launched.

Slice 3 headless measurements: 50 successive target updates took 245 µs median / 288 µs maximum locally, including dispatch and redraw in TestWindow. The test asserts stable preview dimensions, one drag start, one drop and one end. This is not a native renderer or large-tree performance claim.

Slice 1 verification passed: 16 TreeView tests across SDK/look/Studio, 97 ListBox regression tests, `cargo fmt --all -- --check`, and workspace/all-target Clippy. Studio tests and workspace Clippy used `--features test-support,gpui_platform/runtime_shaders` because the local Metal compiler is unavailable.

User follow-up: TreeView now reuses `ScrollbarVisibility::{AutoHide, Hidden, AlwaysVisible}`, with AutoHide as its default. AutoHide reveals on scrolling and hides after the shared 1.25-second idle delay, stays visible throughout scrollbar drags, and preserves tree focus when chrome disappears. Hidden removes its gutter without changing the scroll position. Controls → TreeView includes SDK buttons for all three policies. Follow-up verification passed: 27 TreeView tests across SDK/look/Studio, formatting, and workspace/all-target Clippy with warnings denied. No GUI was launched.

Slice 2 verification passed: all 451 SDK library tests (including 21 TreeView tests), two Shadcn TreeView tests, the Studio fixture test, 97 ListBox regression tests across SDK/look/Studio, formatting, and workspace/all-target Clippy with warnings denied. Studio/workspace commands used `--features test-support,gpui_platform/runtime_shaders`. No GUI was launched.

## Findings from source review

| Finding | Consequence |
| --- | --- |
| `collection/tree_view/control.rs` already uses `ListState` virtualization and expansion motion. | Preserve the renderer; this is not a virtualization rewrite. |
| `set_items` clears selection, active state, and expansion; construction/replacement do not validate unique IDs. | Introduce validated, key-preserving updates before DnD or filtering. |
| `handle_node_select` emits even for unchanged selection and selection mode None; `select_node_by_id` also emits on repeat selection. | Calculate events from before/after state, once per changed aspect. |
| Row click both expands branches and selects them; Enter toggles a branch but selects a leaf. Up/Down wrap, and a unit test asserts wrapping. | Preserve legacy defaults; make new interaction behavior explicit. |
| Expand/collapse action handlers exist, but repository search found no key bindings for them. Right-child navigation uses `idx + 1` without checking depth or enabled state. | Add a TreeView key context and test actual dispatch, including an empty branch followed by a sibling. Do not alter the shared Selector profile for other controls. |
| Hover/press and template callbacks retain flat indices. | Resolve retained interactions by node ID and invalidate transient state across projection changes. |
| `sync_list_state_after_flat_change` skips same-count replacements. Animated collapse eventually calls `rebuild_flat_cache(None, ...)`. | Test measurement invalidation on same-count updates and scroll anchoring at animation completion; splice support alone does not establish correctness here. |
| Both the exposition and gallery card wrap TreeView in an independent `ScrollContainer`. | Fix both consumers. Their scrollbar currently has no connection to the tree's `ListState`. |
| Custom `TreeViewTemplate::render_node` owns handler attachment. | Add a content-only customization path so domain templates do not implement interaction chrome. |
| Tree/root templates do not currently set accessibility roles/states. | Include tree/treeitem semantics in the row-shell work, subject to GPUI support. |
| Only one navigation test covers TreeView control behavior. | Add state and headless event-dispatch coverage before extending interactions. |

The current paths are `controls/collection/tree_view`, not the old navigation path in #77. Compatibility re-exports remain. The issue's `listbox-controls-design2.md` reference is absent locally; use the current ListBox modules and #76 as the reference.

Consumers inspected: Studio control exposition, gallery card, style previews, Shadcn builder and template. Repository consumers use the default template; style previews replace it when the look changes. Public API compatibility still matters for external consumers.

## Proposed contracts

These are design proposals, not APIs already available.

- **Validated state:** a TreeView-specific snapshot/index validates IDs throughout loaded descendants and records parent/sibling relationships. Prepare replacement state before committing it. Keep existing `set_items` as the documented reset path initially; add fallible preserving replacement and validated construction paths. Legacy entry points must reject invalid data before mutation without introducing a panic; document their rejection behavior and prefer the fallible APIs in examples.
- **Reconciliation:** retain selection for existing enabled IDs, including hidden nodes; retain expansion only for surviving branches. Apply initial expansion only to newly introduced branches. Active node and range anchor must be visible and enabled. On collapse, prefer the collapsing ancestor; on filtering/removal, prefer a surviving visible ancestor, then the nearest eligible displayed row, otherwise None.
- **Events:** compute final-state deltas before delivery. Unchanged selection membership emits no selection event even when order or parent changes. Content replacement can still repaint without a selection event. Cross-tree changes finish both state commits before observers receive mutation/lifecycle events.
- **Interaction compatibility:** retain current branch-click, wrapping, and Multiple-toggle defaults. Add an opt-in policy for disclosure-only expansion, bounded navigation, plain-click unchecking, and Extended selection. Studio's new workflow uses disclosure-only expansion and bounded navigation. Keep existing `Single` semantics; do not silently turn it into required selection. Match ListBox terminology where semantics match.
- **New workflow keyboard behavior:** Left collapses or moves to an eligible ancestor; Right expands or moves to an eligible direct child, never a sibling. Space selects/toggles; Enter activates and emits a distinct activation event. Selection-following-active is explicit. Select-all adds visible eligible nodes while preserving hidden selections; explicit clear clears all selected nodes when permitted.
- **Input surfaces:** disclosure, row selection/drag, and embedded SDK controls have distinct boundaries. Dragging must not produce an extra click-selection or expansion. Scope element IDs by tree as well as node. Preserve focus and theme updates with named and inline templates.
- **Filtering:** match loaded domain objects and show matches plus ancestor paths. Initially a matching branch does not automatically include all descendants. Derived expansion is separate from user expansion; clearing the filter restores user choices. No lazy fetching. Host sorting applies only within sibling groups.
- **Projection drops:** initially disable before/after reorder while a filter or custom sort is active. Allow an unambiguous Into proposal only if the host can resolve the parent against current source data. Later add keyed filtered reorder only when a fixture requires it.
- **Positioning:** `scroll_to` minimally reveals; `scroll_to_center` centers with clamping. Smooth counterparts share interruption behavior. These never select or focus. Ordinary requests ignore collapsed/filtered/missing nodes; a separate explicit reveal operation expands loaded ancestors, never overrides filtering or fetches data. New requests and manual input interrupt motion.
- **Scrolling:** `ListState` is authoritative. Preserve hover scrolling by default; expose `require_focus_for_scroll` and enable it in Studio. Unfocused wheel reaches the page, focused wheel is contained at endpoints, clicking away releases focus once, accepted drag auto-scroll does not require focus.

## Developer-facing construction sketch

First implement and review the typed Shadcn builder equivalent. This illustrative local macro would expand to that builder and `.spawn(cx)` during construction, not create entities on each render. Names remain provisional.

```rust,ignore
tree_view! { cx;
    id = "project-workspace";
    look = look.as_ref();
    items = project_nodes; // TreeNode<ProjectEntry>; children express hierarchy
    viewport = { width: 360.0, height: 420.0 };
    indentation = 16.0;
    interaction = explorer_policy;
    branch_content = folder_content; // named content template
    leaf_content = |node, window, cx| {
        document_content(&node.data, window, cx) // inline alternative
    };
}
```

Both callbacks return ordinary GPUI content. SDK mechanics and the Shadcn row shell supply focus, selection, disclosure, drag surfaces, indicators, and accessibility state. Embedded actions use SDK controls and a propagation boundary. Keep the legacy full-template escape hatch. Store content callbacks outside the existing Send/Sync full-template trait if necessary; do not impose additional Send/Sync bounds on UI captures. Retained callbacks must own their captures, unlike ListBox's render-time borrowed callbacks.

## Delivery slices

1. **Validated updates and regression baseline.** Add a small tree-specific state/index module with fallible validation, preserving reconciliation, and final-state event calculation. Wire the existing control to it incrementally. Test duplicate nested IDs, reorder/reparent/removal, disabled nodes, hidden selection, no-op events, and invalid-input atomicity. Preserve current public defaults and expansion rendering. No DnD yet.
2. **One viewport and correct dispatch.** Add a TreeView scroll adapter over `ListState`; connect a look-owned SDK scrollbar and remove both nested scroll shells. Implement the TreeView key context, eligible parent/child traversal, keyed transient interaction, collapse fallback, and anchor/measurement corrections. Prove wheel routing in a nested page before committing to the adapter design. Expose scroll and hover events in Studio.
3. **Content-only templates and single-node DnD (#44).** Add the row shell/content seam and the two-workspace fixture in Luma Studio → Controls → TreeView. Demonstrate named/inline domain templates, then prototype the local construction macro in that exposition. Support before/after/into, sibling reorder, reparenting, root append, and empty destinations. Add basic edge auto-scroll and themed preview/indicators; measure target-update latency and preview stability. Keep host data mutation and business policy in Studio.
4. **Hierarchical projection and positioning.** Add loaded-data filtering, sibling sorting, independent user/derived expansion, immediate positioning, then smooth positioning. Add small filtering and scroll fixtures plus Extended selection/ranges in displayed preorder. Test the explicitly restricted drop policy under projections.
5. **Grouped moves and scale validation.** Normalize selected ancestor/descendant roots; move roots in displayed preorder. Proposed capture policy: include visible selected roots only; their subtrees move intact, while separately hidden selected nodes remain. Verify cross-tree state transfer and large-tree behavior. Promote only helpers actually needed by both controls.

Each slice should be independently reviewable. Slice 1 is the recommended implementation starting point; group moves, async loading, measured rich-content rows, hover-to-expand, and a universal collection abstraction are not prerequisites.

## DnD design and sharing boundary

- Reuse `infra::drag_drop::KeyedDrag`, native binding, scope checks, proposal liveness, and completion guards. Capture node IDs, not flat indices. Represent a destination as tree ID, optional parent ID, sibling anchor, and explicit before/after/into intent. Validate target identity and parent membership again at commit.
- Existing shared proposals carry only `target` and `before`; their completion events classify by source/target equality. Prototype a tree-location wrapper that includes parent information, and define tree-level event mapping explicitly. Same-tree reparenting must not accidentally masquerade as a cross-tree transfer merely because the parent changed. Do not change ListBox event semantics.
- After an expanded branch means after its entire subtree, in its parent's sibling sequence; it never means before its first child. Into means append to that branch initially. Show parent-depth markers and a separate branch highlight so these intents remain distinguishable.
- Validate source existence, accepted scope, destination eligibility, anchor membership, self/descendant cycles, and every ID in the moved subtree. Host policy decides whether leaves can become parents; the first fixture rejects that conversion. Dropping at the current location is a no-op with completion events only.
- Prepare both replacement snapshots and reconciled states before mutating either tree. Revalidate any deferred proposal immediately before commit; apply both in one synchronous host transaction and then publish events. Same-tree moves preserve selection/expansion. For cross-tree moves, transfer surviving subtree selection/expansion explicitly under the destination policy and reconcile source active/anchor state; do not reset the destination's unrelated expansion.
- `ScrollMotion` currently requires `ScrollHandle`; ListBox's edge-scroll controller is private and also handle-specific. Reuse timing/cancellation and the linear edge response through a minimal internal adapter only when required. Keep tree geometry, ancestry, and projection separate from ListBox.
- The installed GPUI `ListState` provides viewport bounds, logical offsets, scrollbar pixel offsets/maxima, scrollbar drag hooks, `scroll_by`, and visible item bounds. Use these APIs instead of adding another scroll state. Its native list wheel listener is unconditional in the bubble phase: copying ListBox's `overflow_hidden` trick is insufficient. The viewport slice needs a headless dispatch spike for routing without consuming the parent page's wheel input; if public hooks cannot express it, isolate a minimal GPUI change as a dependency rather than replacing virtualization.

## Validation

- Pure state tests: deep duplicate IDs, preserving updates, removal/disable fallback, filter restoration, event deduplication, displayed ranges, sibling sorting, and move validation including collisions anywhere in a subtree.
- Headless GPUI tests: real key dispatch, disclosure versus row/child input, Escape/cancellation, one completion event, foreign/stale proposals, focus exit, nested wheel routing and endpoints, scrollbar synchronization, smooth interruption, and unfocused drag auto-scroll.
- Large fixture: 10,000+ loaded nodes with wide/deep branches. Report source count, displayed row count, constructed row count, and template calls separately. Assert bounded element construction, not constant-time source traversal; the existing flat cache clones displayed data. Exercise same-count changes and animation completion while scrolled.
- Run `cargo fmt --all -- --check`, focused TreeView/ListBox/shared-DnD headless tests with `test-support`, Shadcn and Studio fixture tests, and `cargo clippy --workspace --all-targets --features test-support`. Use formatting in write mode for changed Rust files if needed. Do not fix unrelated formatting or lint issues as part of these slices.
- Manual checks use Luma Studio → Controls → TreeView and remain with the user. Keep the fixtures' event trackers, scoped inspectors, and rendering statistics there. Do not launch GUI applications automatically.

The original review used source inspection. Implementation validation uses headless tests, formatting, and Clippy. The local Metal compiler is unavailable, so Studio/workspace verification enables GPUI's existing `gpui_platform/runtime_shaders` feature on the command line; no manifest change or toolchain installation is required. No GUI application has been launched.

# Fixed-height Table measurement optimization

**Tables that declare `visible_row_height(...)` now avoid constructing every offscreen row for measurement.** The public API and existing clipping contract are unchanged. The default Table also gains reliable distant reveal and End-key navigation for unmeasured rows.

This follows the [collection interaction analysis](collection-analysis.md) for [issue 167](https://github.com/gpui-luma-dev/gpui-luma/issues/167). It is a targeted fixed-height optimization; variable-height Tables retain the measured fallback.

## Matched headless results

Both versions use the same keyed six-column grids, Unicode labels, 1100 × 700 viewport and explicit 36-pixel row height. The richer variant adds nested passive text content in every cell. Before captures use the prior SDK implementation with these same fixed-height inputs; the earlier variable-height grid baseline is not substituted as the comparison.

Release measurements were collected October 21, 2026, on the same macOS arm64 machine as the SDK baseline. Each phase has three warmups, five batches of means and a separate allocation pass, with no concurrent builds or profiling. Every phase starts with a fresh test App, performs explicit shutdown/draining afterward, and checks weak control and App release. These are single five-batch comparisons on mock fonts/graphics, not replicated native latency or presentation measurements.

| Fixed-height workload | Rows | First paint ms, before → after | One-value replacement ms, before → after | Column drag/commit ms, before → after |
| --- | ---: | ---: | ---: | ---: |
| Six columns | 1,000 | 26.499 → 2.143 | 26.321 → 1.026 | 28.631 → 3.655 |
| Six columns | 10,000 | 261.325 → 3.284 | 258.236 → 1.370 | 273.031 → 3.791 |
| Richer six columns | 1,000 | 47.881 → 3.711 | 46.252 → 1.814 | 53.370 → 6.531 |
| Richer six columns | 10,000 | 488.904 → 4.857 | 492.633 → 2.154 | 519.083 → 6.588 |

For 10,000 rows, one-value replacement drops about **99.5%** in both grids. The six-column case goes from 1,826,182 to 7,793 allocation calls and from about 1.52 GB to 7.69 MB of requested allocation traffic per replacement. Requested bytes include repeated requests/reallocations; they are not retained heap or resident memory.

Presenter counts explain the change: first paint uses roughly **66 row presenters** rather than more than 10,000; replacement uses **33** rather than 10,019; a column drag/commit uses **109** rather than 10,095. These counts cover all event-induced rendering in one operation, not a count of simultaneously visible rows. Metadata setup, row-vector copying and key validation still involve the loaded dataset; the complete operation is not claimed to be O(viewport).

Warm redraw allocation counts are essentially unchanged (4,296 → 4,298 in the 10,000-row grid). The optimization removes full-dataset presentation work from startup/invalidation rather than adding a cache that changes control ownership.

## Implementation and correctness boundaries

- Fixed heights use GPUI uniform item-height metadata. Scrollbar extents include offscreen rows without evaluating their presenters. Replacements that change the row count seed the new metadata too.
- GPUI clears hints on first layout and viewport-width changes. A post-child-prepaint callback restores fixed heights and requests one bounded redraw; it does not reseed every warm frame or column-width edit. Ordinary remeasurement retains existing hints.
- Default/unrestricted row and cell content can grow. Without an explicit fixed height, fixed-viewport/fill-height Tables continue to use full measurement. No arbitrary custom content is silently clipped or assumed uniform.
- Fill-height auto-sized pages retain their measured current-page behavior because layout stretches the row height. Switching back to smooth scrolling clears the stretched height and restores the fixed-height policy when configured.
- Distant reveal uses accurate height metadata when available. For unmeasured variable-height targets it anchors directly, following the approach already used by TreeView. Fixed-height keyboard movement continues to scroll minimally at a viewport edge.

The fixed border-box and overflow clipping already existed in `visible_row_height`; its documentation now describes that contract and the measurement fast path. No new public method or data-paging API is introduced. All rows are still supplied by the caller. `set_items` still validates/remaps the full dataset, and the default variable-height measurement costs remain an open follow-up.

## Positioning result

The before [collection baseline](collection-baseline-macos-arm64.md) marks default-Table distant reveal as unsuccessful. The new default-Table capture has the same 10,000 loaded records, now with verified successful first/last positioning: [summary](table-fixed-after-default-10000.md), [raw JSONL](table-fixed-after-default-10000.jsonl). Integration tests check that the final row is actually prepainted inside the viewport, including the default configuration; SDK tests also exercise End-key navigation on variable-height content. Historical capture annotations remain unchanged.

## Raw comparisons

- 1,000-row grid: [before summary](table-fixed-before-grid-1000.md), [before JSONL](table-fixed-before-grid-1000.jsonl), [after summary](table-fixed-after-grid-1000.md), [after JSONL](table-fixed-after-grid-1000.jsonl).
- 10,000-row grid: [before summary](table-fixed-before-grid-10000.md), [before JSONL](table-fixed-before-grid-10000.jsonl), [after summary](table-fixed-after-grid-10000.md), [after JSONL](table-fixed-after-grid-10000.jsonl).
- 1,000-row richer grid: [before summary](table-fixed-before-rich-1000.md), [before JSONL](table-fixed-before-rich-1000.jsonl), [after summary](table-fixed-after-rich-1000.md), [after JSONL](table-fixed-after-rich-1000.jsonl).
- 10,000-row richer grid: [before summary](table-fixed-before-rich-10000.md), [before JSONL](table-fixed-before-rich-10000.jsonl), [after summary](table-fixed-after-rich-10000.md), [after JSONL](table-fixed-after-rich-10000.jsonl).

Reproduce a case with `cargo bench --locked -p luma-sdk-perf --bench collection_workloads -- --measure --case table-fixed-grid-6 --size 10000 --output /tmp/table-fixed.jsonl`. Use `table-fixed-rich-grid-6` for the richer variant. The [protocol](collection-protocol.md) describes boundaries and validation.

## Validation and manual review

- 181 SDK collection tests pass (one manual measurement intentionally ignored), including six new regression tests for bounded construction/refresh, exact scroll extents, column/viewport resizing, size/theme/custom content, variable-height fallback, paging transitions, distant reveal and minimal keyboard scrolling.
- Eight profiling-fixture tests pass, now covering the repaired default-Table positioning and both fixed-height benchmark variants.
- Workspace Clippy with all targets/features, formatting, whitespace checks and all 36 Python helper tests pass.
- All 107 retained before/after/default-positioning workload records passed primary-control and App release checks. RSS remains process/allocator history, not native heap ownership.

Studio Controls → Scrolling Table now provides 10,000 fixed-height tasks, selection/reordering, reverse-order replacement, resizable columns, and a jump-to-last-row button. The user manually tested this example and reported it working. All 101 Studio tests pass. Faster scrolling and longer sustained scroll motion are deferred follow-up work. Native runtime tracing is still needed before making an end-user latency claim.

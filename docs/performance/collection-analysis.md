# Table and TreeView: interaction costs and next work

**Table's full-dataset measurement is the first optimization target.** A warm viewport can be cheap while replacing one value, sorting, filtering or resizing a column still rebuilds every row's presentation. TreeView shows a separate, smaller cost in displayed-tree traversal and data/projection rebuilds. Neither finding requires a hypothetical 10,000-item navigation menu.

This follow-up to [issue 167](https://github.com/gpui-luma-dev/gpui-luma/issues/167) adds measurement fixtures and corrects harness shutdown; it does not change production Table or TreeView code. Sidebar remains a documented stress limit, rather than the next optimization priority.

Subsequent implementation: the [fixed-height Table optimization](table-fixed-height-optimization.md) adds a fast path through the existing fixed-row-height contract and fixes unmeasured distant reveal. The analysis and raw baseline below predate that change. Variable-height measurement and whole-dataset update costs remain open.

## Evidence

Collected October 21, 2026, on the same arm64 macOS machine as the [original SDK baseline](sdk-analysis.md), using release optimization, GPUI 1.24.2 and mock fonts/graphics. The root viewport is explicitly bounded to 1100 × 700. Each operation starts with a fresh control and test application, has three warmups, five timed batch means, and a separate allocation pass. No concurrent builds or profiling ran during retained measurements.

- [100 and 10,000 records: summary](collection-baseline-macos-arm64.md), [raw JSONL](collection-baseline-macos-arm64.jsonl): 137 operation measurements.
- [1,000 records: summary](collection-1000-macos-arm64.md), [raw JSONL](collection-1000-macos-arm64.jsonl): 69 operation measurements.
- [100,000 expanded tree nodes](collection-tree-open-100000-macos-arm64.md), [raw JSONL](collection-tree-open-100000-macos-arm64.jsonl), and [100,000 collapsed tree nodes](collection-tree-collapsed-100000-macos-arm64.md), [raw JSONL](collection-tree-collapsed-100000-macos-arm64.jsonl): 24 additional measurements in isolated processes.
- [Protocol and reproduction](collection-protocol.md) define every phase, data shape, ownership boundary, counter and limitation. All **230 primary-control and App release checks** passed.

These are CPU costs of complete headless operations, not native input latency or GPU presentation. Each result is one five-batch run, not a replicated statistical comparison. Costs from different table configurations and tree visibility states must not be treated as interchangeable.

## Table: viewport redraw hides replacement and resize work

Tables use stable row keys and default SDK row chrome. The grid adds six visible text columns; the richer grid uses nested passive text content in each cell. The explicit fill-height flag activates a different GPUI measurement policy from the default configuration.

| Workload | Warm redraw ms | Replacement ms | One-value replacement ms | Column drag/commit ms |
| --- | ---: | ---: | ---: | ---: |
| Default keyed Table, 1,000 rows | 0.156 | 0.268 | 0.268 | — |
| Explicit fill-height Table, 1,000 rows | 0.161 | 5.650 | 5.692 | — |
| Six-column fill-height grid, 1,000 rows | 0.689 | 25.286 | 25.491 | 31.048 |
| Richer six-column grid, 1,000 rows | 1.247 | 49.881 | 49.717 | 56.395 |
| Default keyed Table, 10,000 rows | 0.160 | 0.642 | 0.620 | — |
| Explicit fill-height Table, 10,000 rows | 0.205 | 62.215 | 63.228 | — |
| Six-column fill-height grid, 10,000 rows | 1.218 | 261.010 | 259.649 | 273.139 |
| Richer six-column grid, 10,000 rows | 2.664 | 492.580 | 494.991 | 496.499 |

The 10,000-row grid invokes roughly **18 row presenters for warm redraw**, but **10,018 for replacement** and **10,090 per column drag/commit**. Fresh first paint also invokes more than 10,000 presenters. The amplification comes from offscreen measurement, not from 10,000 rows being visibly painted.

Source inspection explains the path:

1. `crates/sdk/src/controls/collection/table/control/mod.rs::from_builder` opts into GPUI `ListState::measure_all()` when `visible_rows` is set or `fill_height` is enabled. The source comment explicitly ties this to correct scrollbar extents.
2. `TableControl::set_items` validates keys, remaps selection and remeasures the list, even for one value delivered through full replacement.
3. `control/resize.rs::handle_column_resize_move` updates column geometry and calls `remeasure`. GPUI's measure-all path then invokes all row presenters, including every cell in a grid.

For the 10,000-row grid, replacement requests about **1.52 GB of allocation traffic per operation**; the richer grid requests about **3.23 GB**. Those figures include allocation/reallocation requests and repeat traffic. They are neither retained heap nor simultaneous resident memory, and must not be described as a multi-gigabyte data model.

Sorting/filtering ownership also matters: Table's owner computes a sorted/filtered vector and calls `set_items`. The benchmark includes that owner work, but full row measurement dominates the explicit fill-height grid. One changed value delivered through the existing replacement API costs almost as much as replacing the whole dataset.

### Distant positioning is a correctness boundary

The default Table's `scroll_to_row(last)` does not reveal an unmeasured final row in this fixture. The source calls GPUI `scroll_to_reveal_item`, which derives a target from measured height sums; unmeasured rows do not yet contribute full extents. TreeView already uses a direct keyed anchor when a target is outside its measured region. The measured fill-height Table succeeds because all rows have extents.

Consequently, the summaries mark the default Table `jump` values with an asterisk: they time an unsuccessful API request, **not successful positioning latency**. Other Table configurations and both TreeView states have an integration check that the last loaded target is actually prepainted inside the viewport. The default configuration still passes real wheel movement and keyboard navigation checks.

Do not simply remove `measure_all()`. A viable optimization must retain scrollbar range, distant reveal, header/body alignment and variable-height correctness.

## TreeView: loaded hierarchy and displayed hierarchy differ

The forest has up to ten folders per root and nine leaves per folder, with depth two below each root. Counts include folders and leaves. All data is loaded; this does not model lazy fetching. At 100,000 nodes the collapsed forest displays about 991 roots, while the expanded forest displays all loaded nodes.

| Tree state | Loaded nodes | Warm redraw ms | Replacement ms | Sibling sort ms | Filter ms | Reveal/jump ms | Toggle one 100-descendant subtree ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Expanded | 1,000 | 0.294 | 0.755 | 0.727 | 0.467 | 1.066 | 0.576 |
| Collapsed | 1,000 | 0.117 | 0.212 | 0.266 | 0.467 | 0.795 | 0.176 |
| Expanded | 10,000 | 0.669 | 3.363 | 2.728 | 1.072 | 2.984 | 2.077 |
| Collapsed | 10,000 | 0.260 | 1.203 | 0.708 | 0.993 | 0.919 | 0.340 |
| Expanded | 100,000 | 5.597 | 37.014 | 31.128 | 10.207 | 29.579 | 22.203 |
| Collapsed | 100,000 | 0.314 | 9.185 | 3.352 | 8.246 | 2.690 | 0.502 |

The expanded 100,000-node tree still invokes only **23 presenters per warm redraw**. Its increasing warm CPU time therefore does not establish failed row virtualization. Source paths that remain outside viewport virtualization include:

- `control.rs::render` synchronizes branch transitions and calls `desired_flat_count`, which visits the displayed projection each render.
- `replace_items` validates and rebuilds the index/projection, reconciles identities, rebuilds the flat cache and resets list measurements. A root label/data edit delivered through this API still takes that full path.
- Filtering and stable sibling sorting rebuild `Projection` over the loaded hierarchy, retain ancestor paths and reconcile state.
- Expansion/collapse rebuilds the flat cache of the displayed hierarchy, even when only one small subtree changes. Keyed reveal performs visibility/path work and subsequent layout correction.

These are investigation boundaries, not per-symbol CPU attribution. Expanded 100,000-node trees are a stress case; the sub-millisecond warm results and roughly 1–3 ms updates at smaller sizes do not support an urgent general rewrite of TreeView.

## Lifecycle and memory interpretation

An exploratory sequential 100,000-row Table run was stopped before the richer grid after RSS grew above 70 GiB. That run lacked explicit TestAppContext shutdown and is excluded from retained baseline results. GPUI's `add_window_view` registers a visual context for `on_quit` cleanup; dropping the outer context alone retains it. Shutdown must also drain pending tasks. The corrected runner verifies both weak control release and weak App release after shutdown. The original SDK runner now performs that cleanup too.

The corrected 100/10,000 matrix's largest sampled RSS after App shutdown was about **5.48 GiB**; the isolated 100,000-node expanded tree run reached about **831 MiB** after shutdown. RSS still includes allocator retention and process-global history even after App release, and the matrix does not identify native heap ownership. No native collection leak is established. Exploratory incomplete data remain ignored under `target/perf-traces/issue-167/collections-exploratory`; the summary rejects their missing footer.

Default sizes exclude 100,000-row Table stress, and the runner stops if RSS exceeds 8 GiB after phase shutdown. Isolated case/process measurements should precede any larger rich-cell run. Historical SDK sequential RSS is now explicitly annotated as including harness retention.

## Ranked implementation boundaries

1. **Table measurement and distant reveal.** Start with fixed-height/default text rows: evaluate GPUI's existing uniform-height hint support or cached measurements so startup/refresh/resize does not construct all offscreen cells. Keep full/variable-height fallback where required. Add correctness tests for scrollbar extent, bottom reveal, viewport/column resizing, custom heights, text/font/theme changes, selected identities and paging. Re-run the 1,000/10,000-row grids and collect native traces before making a responsiveness claim.
2. **TreeView steady traversal and small expansion.** Determine whether displayed counts and settled transition work can be maintained by revisions rather than recomputed on every render. Preserve simultaneous animation, projection, hidden selection, active fallback and scroll anchors. Compare expanded and collapsed trees, not just the worst-case expanded stress fixture.
3. **Update/projection cost and keyed selection.** Attribute index validation, identity remapping and row-data copying before adding patch APIs. Table's stable-key selection currently constructs a lookup over all row keys per call; decide whether repeated keyed selection justifies a retained index and its memory cost. Explore narrower updates only with explicit state semantics.
4. **Complete interaction and ownership coverage.** Native wheel/key/resize traces, application presentation and input latency, interactive/variable-height cells, many selected rows, deep/lazy trees, row drag/drop, paging and animated settling remain open. Add isolated memory attribution rather than extrapolating from allocation traffic or RSS.

The current evidence supports a small Table-focused optimization first. Sidebar virtualization does not precede this work without a representative usage problem.

## Validation

- 175 SDK collection tests passed (one manual measurement remains intentionally ignored).
- Eight profiling-fixture tests passed: the existing composed-panel check plus seven collection integration checks.
- All 35 Python helper tests passed; every retained collection JSONL validates and regenerates its summary.
- Workspace Clippy with all targets/features and formatting checks passed.
- The original eight-case SDK benchmark smoke run passed after adding explicit App shutdown.

All measurements in this report are headless. No new native responsiveness or heap-ownership claim is made, and no PR has been created for this analysis branch.

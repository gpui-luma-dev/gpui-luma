# Table and TreeView interaction protocol

This extends [issue 167](https://github.com/gpui-luma-dev/gpui-luma/issues/167) beyond the original fixed-viewport collection redraw. It measures complete public-API operations and real headless input dispatch. No production control or public API is changed.

## Run

```sh
cargo test --locked -p luma-sdk-perf --test collection_fixtures
cargo bench --locked -p luma-sdk-perf --bench collection_workloads -- --measure --size 100 --output /tmp/collections-smoke.jsonl
cargo bench --locked -p luma-sdk-perf --bench collection_workloads -- --measure --output /tmp/collections.jsonl
python3 -B scripts/summarize_collection_perf.py /tmp/collections.jsonl --output /tmp/collections.md
```

Use `--case table-grid-6 --size 10000` for an isolated process. Supported sizes are 100–100000; the default matrix is 100 and 10000. Use 100000 only as an explicit isolated stress case. The runner stops without a completion footer if process RSS exceeds 8 GiB after a phase has shut down. Measurements must run without concurrent builds or profiling. The benchmark uses GPUI TestAppContext, an explicitly bounded 1100 × 700 root, mock fonts and graphics, and release optimization. It launches no GUI application. The SDK test-support feature supplies test instrumentation, not a native platform.

## Fixtures

- `table-default`: keyed data with default Table geometry; no explicit `fill_height` flag.
- `table-fill`: same data and default row chrome with explicit `fill_height`.
- `table-grid-6`: fill-height Table with six 150-pixel columns, SDK header/row chrome, and column resizing enabled.
- `table-rich-grid-6`: same grid with two text fragments and nested layout per cell, representing custom presenter composition. These cells are passive content, not replacement controls. There are no embedded buttons or editors in this fixture.
- `table-fixed-grid-6` and `table-fixed-rich-grid-6`: matching grids with the existing `visible_row_height(36.0_f32)` contract. All rows have a 36-pixel border box and clip overflow; these cases compare the uniform-height measurement path.
- `tree-open` and `tree-collapsed`: loaded forests with identical data, up to ten folders per root and nine leaves per folder, depth two below the root. Each complete root subtree contains 101 nodes; partial tails preserve the exact requested count. All loaded branches start expanded or collapsed respectively. Tree motion is disabled.

Every record has a stable string key, Unicode label, integer ID, and small numeric payload. Table's owner retains canonical source rows for replacements. Tree nodes retain their source hierarchy. Counts are loaded records/nodes, not necessarily displayed rows. At 100 nodes the collapsed tree has only one displayed root, so keyboard and selection phases are skipped. Collapsed trees with at most 22 displayed roots fit this default-theme viewport, so their wheel phase is skipped; this includes the 100- and 1,000-node cases.

Counters observe one Table row-label callback or first-column cell callback per presenter invocation, and each Tree branch/leaf content callback. These include offscreen measurement, not just painted rows. Atomic counters add a small fixed cost and are retained consistently across cases. Integration tests verify bounded steady presenter work, actual wheel movement, keyboard active-row changes, stable selection after replacement/sorting, filter results, nested expansion and committed column geometry.

## Phases

Each phase starts with a fresh TestAppContext and control, so filtering, sorting or scrolling from one phase cannot contaminate another. Control construction, first render, simulated resize, initial focus and active row are outside the timed phase except where expressly measured below.

| Phase | Included work |
| --- | --- |
| `construct_drop` | Build and drop an additional unpainted control, including data generation, validation/key configuration and executor drain; the displayed fixture stays alive. |
| `first_paint` | Build and display a fresh control, release the previous one, then drain layout/rendering. Includes construction and replacement cleanup, not just paint. |
| `warm_redraw` | Notify the control and drain rendering without changing data. |
| `wheel` | Dispatch a real wheel event inside the control, alternating ±250 pixels, and drain. |
| `keyboard` | Dispatch alternating Down/Up keystrokes to the focused control and drain. Includes focus/selection/reveal handling, not OS input scheduling. |
| `selection` | Table: select one stable key, including its key-to-index lookup. Tree: select another displayed root ID. Does not represent a pointer click or a large selected set. |
| `replace` | Table: clone the owner's source vector and call `set_items`. Tree: clone the current hierarchy and call state-preserving `replace_items`. |
| `single_value_replace` | Same public replacement APIs after editing one value in a Table row, or one Tree root's label/data. This measures one-record edits delivered through full replacement; it is not an incremental patch API. |
| `sort` | Table: owner clones and stably sorts rows by a numeric field before `set_items`. Tree: change its stable sibling comparator. These are different ownership boundaries, not interchangeable sorting implementations. |
| `filter` | Table: owner clones and retains alternating 50% subsets before `set_items`. Tree: install a 10%-matching predicate, including automatic ancestor paths. Selectivity and hierarchy differ. |
| `jump` | Table: alternate first/last row positioning. Tree: reveal loaded first/last node by key, opening ancestors as needed. Includes three simulated frame drains for measured positioning. After warmup the last path is already open. |
| `expand_collapse` | Toggle one 101-node root subtree, exposing or hiding up to 100 descendants. The rest of the loaded forest remains unchanged. |
| `column_resize` | Actual mouse-down, drag moves and mouse-up on the first column divider, alternating ±24 pixels. Includes event dispatch and resulting layout work for one drag step/commit, not an entire continuous user gesture. |

Release builds omit GPUI debug selectors. Column drag coordinates therefore come from this fixture's fixed column widths and public default Table theme geometry. Debug-profile integration tests verify that these gestures change the real measured column width. They do not substitute setters for drag input.

## Samples, correctness and boundaries

Each phase has three warmups, five batches of means, and a separate measuring-thread allocation pass. Iterations per batch are five at 100 records and two at 10000/100000. Timing records retain all five means, median/range, allocation calls, requested bytes, operation/presenter counters, and current process RSS before/after release. Counters span all passes, so divide by the recorded operation count to compare per-operation presenter work.

After each phase the control and owner source rows are removed, executor work is drained and three frames flush buffers. A weak reference must show the primary control released. The test application is then explicitly quit, its pending tasks drained, and its outer context dropped; a second weak reference must confirm that the App itself released. Shutdown occurs outside the timed operation. GPUI's `add_window_view` registers its visual context for `on_quit` cleanup; dropping the outer TestAppContext alone does not perform that cleanup. These probes do not inspect every nested entity or subscription. RSS before release, after control removal, and after App shutdown includes allocator, GPUI caches and the sequential matrix's process history; current RSS is not a peak, heap ownership, or proof of a leak. For ownership attribution use isolated processes and a working native heap profiler.

Schema-2 JSONL records include the complete case/size manifest, revision/dirty state, build assertion mode, Rust/OS/architecture and capture time. The summary rejects missing phases, duplicate/unknown workloads, invalid batches/counters, failed release checks and incomplete footers. The original SDK baseline summary uses a different schema and remains unchanged. Current captures include `table_default_distant_reveal_verified: true` after integration checks confirm the repaired positioning path; historical captures without it retain the unsuccessful-jump annotation.

These numbers describe headless CPU work, not native wheel latency, GPU presentation, browser performance or end-user frame rates. Run native interaction traces, presentation and visual checks before claiming native responsiveness improvements. Real interactive cells, variable-height rows, large multi-selection, lazy-loaded trees, deep chains, drag/drop, paging, theme changes and animated expansion remain separate coverage gaps.

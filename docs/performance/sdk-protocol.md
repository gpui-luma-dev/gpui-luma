# SDK performance analysis protocol

Issue [167](https://github.com/gpui-luma-dev/gpui-luma/issues/167) tracks a performance map for the SDK, both look adapters, color controls, and plotting. This first baseline provides shared measurement tooling and representative composed workloads. It does not complete the inventory-only scenarios in [the coverage matrix](sdk-coverage.csv).

## Repeatable workloads

`apps/sdk-perf` shares the same scene constructors between a headless benchmark and an opt-in native executable. Every interactive control uses SDK builders or look factories. Scene layout is noninteractive GPUI layout; no alternative control implementations are benchmarked.

| Scene | Size means | Primary controls | Current cases |
| --- | --- | --- | --- |
| forms | Repetitions of seven controls | 7 × size | Button, Checkbox, Switch, TextField, TextArea, Slider, Progress; SDK, Shadcn and Radix |
| navigation | Sidebar rows and selector/menu items | 7 | Sidebar, up to 20 tabs, up to 100 accordion sections, Stepper, Pager, closed PopupMenu and Selector |
| collections | Records per collection | 2 | Flat TreeView and default Table |
| layout | Frames | size + 1 | Frames and a SplitView with an SDK button in its sidebar |
| color | Pairs of controls | 2 × size | Vector ColorField and HueRing |
| plot | Samples | 1 | Cartesian sine-wave line chart |

Motion is disabled for steady-state animated controls. Primary counts exclude internal child entities and bounded viewport wrappers. Table and TreeView have separate visible 580 × 700 panels; SplitView occupies a 450 × 180 panel. A headless regression verifies that the collection panels fit and do not overlap. Practical look comparisons use each adapter's defaults, rather than forcing identical appearance. Construction also initializes application fixture vectors and closures; the results are composed-scene costs, not intrinsic per-control costs.

## Headless execution

```sh
cargo bench --locked -p luma-sdk-perf --bench sdk_workloads -- --measure --output /tmp/sdk-baseline.jsonl
python3 -B scripts/summarize_sdk_perf.py /tmp/sdk-baseline.jsonl --output /tmp/sdk-baseline.md
```

For a quick correctness check across all scenes and looks:

```sh
cargo bench --locked -p luma-sdk-perf --bench sdk_workloads -- --measure --size 2 --cycles 0 --output /tmp/sdk-smoke.jsonl
python3 -B scripts/summarize_sdk_perf.py /tmp/sdk-smoke.jsonl
```

Use `--scene forms --look shadcn --size 100` to isolate one workload. A separate process per workload is preferable when attributing RSS. Default sizes are 10/100/1000 form repetitions and frames, 100/10000 navigation/collection items, 1/10/100 color pairs, and 100/10000/100000 plot samples. `--cycles` controls repeated create/paint/remove cycles (default 10, maximum 1000; zero skips these cycles). Run measurements sequentially without concurrent builds.

The headless platform uses mock font/shaping and graphics services. The benchmark sends a simulated resize event and verifies GPUI's actual viewport is 1200 × 800. It asserts that the warm-redraw workload really renders, and checks tracked control entity release after removal, executor draining, and two additional frames. The fixtures exercise production rendering and public update methods; they do not model native GPU execution.

### Phase boundaries

- `construct_drop`: construct and release an unpainted scene, including listener/subscription setup and executor draining. The displayed scene stays in place.
- `replace_first_paint`: replace the displayed scene, release its predecessor, then perform first layout/prepaint/paint. This includes construction and old-scene cleanup; it is not isolated paint time or process startup.
- `warm_redraw`: explicitly invalidate every tracked root and the scene, then drain rendering work.
- `data_update`: change text, checkbox state, slider/ring values, selector selection, tree selection, or color hue, as applicable. Mutation setup is included. Scenes without implemented updates omit this phase.
- `theme_revision`: bump the SDK app-global theme epoch and process observer fanout. This does not edit tokens, parse CSS, or switch light/dark mode.
- `executor_idle_drain`: drain an already parked test executor. This is not native idle CPU usage.

Shared infrastructure microbenchmarks separately measure layout-scale computation, theme-token cloning, and spring construction plus four deterministic trajectory samples. They do not measure concurrent animation scheduling.

### Output and interpretation

JSONL records include OS/architecture, Rust version, commit, dirty status, capture time, viewport, build assertion mode, workload identity, iteration count, and all five batch means. Timing uses three warmups followed by five batches; the reported median is the median of batch means, not an individual-frame percentile. A separate pass counts measuring-thread allocation calls, including zeroed allocations and reallocations, and requested byte traffic. This excludes background-thread allocations, allocator overhead, retained heap size, and GPU memory.

Historical caveat: the original SDK baseline predates explicit TestAppContext shutdown in the runner. Its sequential RSS also includes retained headless harness contexts, and must not be used to attribute SDK ownership or leaks. The collection follow-up adds App shutdown/draining and weak App-release checks.

Resident bytes are current process RSS (Mach task information on macOS, `/proc/self/status` on Linux; unavailable elsewhere). Memory checkpoints occur after complete lifecycle cycles. The default matrix runs sequentially in one process, so allocator and process-global cache history affects later cases. RSS is not per-control ownership, and growth alone does not establish a leak. Weak probes track the added primary controls, not every SDK-internal entity or subscription.

A completion record is written only after all selected fixtures succeed. The summary tool rejects partial runs, duplicate workloads, invalid sample data, and mismatched completion counts. Correctness tests run in Cargo's normal test profile; existing GPUI interaction tests depend on debug selectors unavailable in release builds.

## Native execution

Building does not launch an application:

```sh
cargo build --locked --release -p luma-sdk-perf --features native
```

After explicit GUI launch approval under `AGENTS.md`, an example capture is:

```sh
xcrun xctrace record --template 'Time Profiler' --time-limit 15s --output /tmp/forms-shadcn.trace --target-stdout /tmp/forms-shadcn.log --launch -- "$PWD/target/release/luma-sdk-perf" forms shadcn 100 --drive
```

Without a mode flag, the fixture remains interactive. `--idle` leaves the scene unchanged for twelve seconds and then quits. With `--drive`, it continuously mutates and redraws the scene for twelve seconds, then quits. It registers the same bundled fonts and Lucide assets used by existing example apps. This is a deterministic stress sequence, not a simulated user typing or pointer event stream. Frame callbacks are scheduling observations, not GPU presentation or input-to-paint timestamps.

Export CPU samples with `xcrun xctrace export --input <trace> --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-profile"]' --output /tmp/cpu.xml`, then run `python3 -B scripts/summarize_sdk_cpu.py /tmp/cpu.xml`. The summary filters to the fixture process and steady trace seconds 2–12.

Capture CPU, Allocations, and Metal/Animation Hitches traces separately to identify instrumentation overhead. Retain traces outside the Git diff under `target/perf-traces/issue-167`, with workload arguments and tool versions. Attribute findings to the launched fixture process; do not treat system-wide activity as SDK work.

## Coverage and next stages

1. Complete inventory-only control families and interaction scenarios in the CSV. Opening/closing overlays, filtering, virtualized scrolling, drag/drop, focus/IME, actual theme changes, and animated settling need dedicated scenarios.
2. Extract native CPU attribution, heap ownership/peak memory, presentation frame distributions, and event-to-paint latency. Use display-specific frame budgets; no headless result establishes a native frame-rate target.
3. Reuse fixtures on supported native platforms. WASM compilation is a compatibility check; browser runtime, scheduling, shaping, and graphics need separate measurements.
4. Rank confirmed findings by user impact, SDK prevalence, scaling, implementation effort, and correctness risk. Keep optimization PRs small and compare unchanged workloads before and after.
5. Put deterministic behavioral/operation-count/allocation regressions in ordinary CI. Keep hardware-sensitive timing trends in periodic benchmarks, without hard wall-clock thresholds in normal tests.

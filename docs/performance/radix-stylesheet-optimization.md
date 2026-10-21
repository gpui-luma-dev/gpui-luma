# Radix form stylesheet reads

Radix form adapters now use an internal immutable `Arc<CommonStylesheet>` snapshot instead of deep-cloning the complete stylesheet on each read. The public owned getter and setter signatures remain unchanged. A setter replaces the snapshot and bumps the revision; cloned Looks observe edits, while forks remain independent. Outstanding readers safely retain the previous snapshot until their resolution ends. No revision cache can become stale.

The change covers Button, Checkbox, Switch, TextField, TextArea, Slider, Progress, shared label typography, and motion-default resolution. Other adapter call sites still use the public owned getter and can be evaluated separately. TextField and TextArea each reuse one snapshot for helper and control geometry. This preserves the existing notification contract and spawn-time motion semantics.

## Matched headless workload

Measured October 21, 2026 on the same machine and settings as the [SDK baseline](sdk-protocol.md): 100 Radix form repetitions, 700 primary controls, 1200 × 800 mock viewport. Five batch means per phase, with allocation traffic measured separately. The retained before/after runs had no concurrent builds or profiling. An earlier after run overlapped a native capture and is excluded.

| Phase | Before ms | After ms |
| --- | ---: | ---: |
| Construct/drop | 4.569 | 2.944 |
| Replace and first paint | 50.244 | 29.647 |
| Warm redraw | 22.670 | 12.500 |
| Data update | 23.969 | 15.099 |
| Theme epoch notification | 22.328 | 13.211 |

Warm redraw time decreased about **45%**, allocations from **483,275 to 99,175 (79%)**, and requested allocation bytes from **76,218,678 to 38,907,178 (49%)**. These are composed-scene CPU measurements, not native frame durations or universal speedups. Timing comes from one five-batch run per version, not replicated trials.

All 700 tracked control entities released in each run. RSS after ten lifecycle cycles was 95,502,336 bytes before and 96,124,928 after (about 0.6 MiB higher). Lower allocation traffic does not imply lower retained memory; these isolated runs do not establish a meaningful RSS improvement or regression. A retained snapshot can temporarily extend an old stylesheet's lifetime during concurrent edits.

Raw results: [before](radix-stylesheet-before.jsonl), [after](radix-stylesheet-after.jsonl).

## Native attribution

Separate Time Profiler captures use the same release form scene and continuously update it for twelve seconds. Analysis uses trace seconds 2–12. Estimated main-thread running CPU was **99.99% before and 91.56% after**. `CommonStylesheet::clone` had **32.73% inclusive weight before** and is absent from the after capture's top fifteen SDK-inclusive symbols. This is consistent with removing deep clones from the measured form paths; it does not establish that all stylesheet clones across the SDK are gone.

Driver callbacks increased from 448 in 12.024 seconds to 730 in 12.246 seconds. The driver follows native frame scheduling, so work throughput differs between versions. These counts are not presented FPS; CPU use at different throughput is not a fixed-work latency comparison. The baseline executable stops on a frame callback, whereas the rebuilt executable uses the independent shutdown timer documented in the profiling app. Neither termination mechanism affects the intended 2–12 second analysis window. Native samples are one capture per version and do not support a statistical latency claim.

[CPU samples summary and capture paths](radix-stylesheet-native-cpu.json). Trace bundles remain ignored under `target/perf-traces/issue-167`. An initial baseline capture overlapped compilation and is excluded.

## Validation

- 242 Radix unit tests and 16 doctests pass, including new snapshot-lifetime/owned-copy/fork and live geometry tests across light/dark mode.
- Workspace Clippy with all targets/features passes; formatting and diff whitespace checks pass.
- Native release fixture builds and both retained captures terminate successfully.
- Existing slider, switch, tabs, typography, motion, palette, and fork regressions remain enabled.

No PR has been created. The broader SDK measurement gaps remain tracked by issue 167.

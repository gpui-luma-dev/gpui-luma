# Collection interaction measurements

Headless release CPU milliseconds; medians of five batch means. Mock fonts/graphics; not native input latency.

| Case | Loaded records/nodes | warm_redraw | wheel | keyboard | replace | single_value_replace | sort | filter | jump | expand_collapse | column_resize |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| table-fixed-grid-6 | 10000 | 1.209 | 2.455 | 1.215 | 255.037 | 258.236 | 254.005 | 136.582 | 1.464 | — | 273.031 |

## Presenter work

Counters include warmups, timed batches, and the separate allocation pass. Initial setup includes construction, first layout, resize, and focus; operation counts normalize all passes.

| Case | Size | First-paint presenter calls/op | Warm presenter calls/op | Warm allocations/op |
| --- | ---: | ---: | ---: | ---: |
| table-fixed-grid-6 | 10000 | 10038.0 | 19.0 | 4296.0 |

All 12 per-workload primary-entity and App release checks passed. RSS includes sequential process history and owner data; it is not an ownership or leak report.

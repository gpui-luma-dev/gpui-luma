# Collection interaction measurements

Headless release CPU milliseconds; medians of five batch means. Mock fonts/graphics; not native input latency.

| Case | Loaded records/nodes | warm_redraw | wheel | keyboard | replace | single_value_replace | sort | filter | jump | expand_collapse | column_resize |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| table-default | 10000 | 0.151 | 0.318 | 0.154 | 0.609 | 0.610 | 0.781 | 0.449 | 0.157 | — | — |

## Presenter work

Counters include warmups, timed batches, and the separate allocation pass. Initial setup includes construction, first layout, resize, and focus; operation counts normalize all passes.

| Case | Size | First-paint presenter calls/op | Warm presenter calls/op | Warm allocations/op |
| --- | ---: | ---: | ---: | ---: |
| table-default | 10000 | 33.3 | 19.0 | 1128.0 |

All 11 per-workload primary-entity and App release checks passed. RSS includes sequential process history and owner data; it is not an ownership or leak report.

# Collection interaction measurements

Headless release CPU milliseconds; medians of five batch means. Mock fonts/graphics; not native input latency.

| Case | Loaded records/nodes | warm_redraw | wheel | keyboard | replace | single_value_replace | sort | filter | jump | expand_collapse | column_resize |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| table-fixed-rich-grid-6 | 10000 | 1.157 | 2.381 | 1.161 | 2.155 | 2.154 | 2.329 | 2.002 | 1.177 | — | 6.588 |

## Presenter work

Counters include warmups, timed batches, and the separate allocation pass. Initial setup includes construction, first layout, resize, and focus; operation counts normalize all passes.

| Case | Size | First-paint presenter calls/op | Warm presenter calls/op | Warm allocations/op |
| --- | ---: | ---: | ---: | ---: |
| table-fixed-rich-grid-6 | 10000 | 66.0 | 19.0 | 7262.0 |

All 12 per-workload primary-entity and App release checks passed. RSS includes sequential process history and owner data; it is not an ownership or leak report.

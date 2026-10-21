# Collection interaction measurements

Headless release CPU milliseconds; medians of five batch means. Mock fonts/graphics; not native input latency.

| Case | Loaded records/nodes | warm_redraw | wheel | keyboard | replace | single_value_replace | sort | filter | jump | expand_collapse | column_resize |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| table-default | 1000 | 0.156 | 0.321 | 0.163 | 0.268 | 0.268 | 0.288 | 0.258 | 0.194* | — | — |
| table-fill | 1000 | 0.161 | 0.329 | 0.167 | 5.650 | 5.692 | 5.612 | 3.005 | 0.167 | — | — |
| table-grid-6 | 1000 | 0.689 | 1.421 | 0.700 | 25.286 | 25.491 | 26.473 | 13.934 | 0.708 | — | 31.048 |
| table-rich-grid-6 | 1000 | 1.247 | 2.493 | 1.226 | 49.881 | 49.717 | 50.721 | 24.810 | 1.249 | — | 56.395 |
| tree-open | 1000 | 0.294 | 0.612 | 0.309 | 0.755 | 0.755 | 0.727 | 0.467 | 1.066 | 0.576 | — |
| tree-collapsed | 1000 | 0.117 | — | 0.120 | 0.212 | 0.212 | 0.266 | 0.467 | 0.795 | 0.176 | — |

*Default Table distant reveal did not reach the unmeasured target in this capture. Its jump value measures an unsuccessful API request, not successful positioning latency; see collection-analysis.md.*

## Presenter work

Counters include warmups, timed batches, and the separate allocation pass. Initial setup includes construction, first layout, resize, and focus; operation counts normalize all passes.

| Case | Size | First-paint presenter calls/op | Warm presenter calls/op | Warm allocations/op |
| --- | ---: | ---: | ---: | ---: |
| table-default | 1000 | 32.6 | 19.0 | 1125.0 |
| table-fill | 1000 | 1019.6 | 19.0 | 1125.0 |
| table-grid-6 | 1000 | 1036.0 | 18.0 | 4092.0 |
| table-rich-grid-6 | 1000 | 1036.0 | 18.0 | 6900.0 |
| tree-open | 1000 | 62.0 | 23.0 | 1449.0 |
| tree-collapsed | 1000 | 20.0 | 10.0 | 834.0 |

All 69 per-workload primary-entity and App release checks passed. RSS includes sequential process history and owner data; it is not an ownership or leak report.

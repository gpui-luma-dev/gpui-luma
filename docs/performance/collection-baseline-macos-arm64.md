# Collection interaction measurements

Headless release CPU milliseconds; medians of five batch means. Mock fonts/graphics; not native input latency.

| Case | Loaded records/nodes | warm_redraw | wheel | keyboard | replace | single_value_replace | sort | filter | jump | expand_collapse | column_resize |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| table-default | 100 | 0.151 | 0.312 | 0.157 | 0.235 | 0.217 | 0.237 | 0.234 | 0.161* | — | — |
| table-fill | 100 | 0.151 | 0.311 | 0.154 | 0.675 | 0.676 | 0.674 | 0.427 | 0.158 | — | — |
| table-grid-6 | 100 | 0.637 | 1.320 | 0.640 | 2.932 | 2.932 | 2.933 | 1.862 | 0.652 | — | 5.561 |
| table-rich-grid-6 | 100 | 1.128 | 2.304 | 1.125 | 5.687 | 5.601 | 5.492 | 3.310 | 1.145 | — | 10.214 |
| tree-open | 100 | 0.227 | 0.472 | 0.229 | 0.364 | 0.357 | 0.371 | 0.228 | 0.711 | 0.367 | — |
| tree-collapsed | 100 | 0.020 | — | — | 0.028 | 0.030 | 0.024 | 0.229 | 0.194 | 0.082 | — |
| table-default | 10000 | 0.160 | 0.312 | 0.167 | 0.642 | 0.620 | 0.805 | 0.468 | 0.197* | — | — |
| table-fill | 10000 | 0.205 | 0.405 | 0.205 | 62.215 | 63.228 | 61.083 | 31.277 | 0.208 | — | — |
| table-grid-6 | 10000 | 1.218 | 2.725 | 1.369 | 261.010 | 259.649 | 263.493 | 133.788 | 1.443 | — | 273.139 |
| table-rich-grid-6 | 10000 | 2.664 | 5.687 | 2.789 | 492.580 | 494.991 | 492.887 | 244.503 | 2.834 | — | 496.499 |
| tree-open | 10000 | 0.669 | 1.356 | 0.667 | 3.363 | 3.311 | 2.728 | 1.072 | 2.984 | 2.077 | — |
| tree-collapsed | 10000 | 0.260 | 0.532 | 0.260 | 1.203 | 1.197 | 0.708 | 0.993 | 0.919 | 0.340 | — |

*Default Table distant reveal did not reach the unmeasured target in this capture. Its jump value measures an unsuccessful API request, not successful positioning latency; see collection-analysis.md.*

## Presenter work

Counters include warmups, timed batches, and the separate allocation pass. Initial setup includes construction, first layout, resize, and focus; operation counts normalize all passes.

| Case | Size | First-paint presenter calls/op | Warm presenter calls/op | Warm allocations/op |
| --- | ---: | ---: | ---: | ---: |
| table-default | 100 | 32.6 | 19.0 | 1122.0 |
| table-fill | 100 | 119.6 | 19.0 | 1122.0 |
| table-grid-6 | 100 | 136.0 | 18.0 | 4089.0 |
| table-rich-grid-6 | 100 | 136.0 | 18.0 | 6897.0 |
| tree-open | 100 | 62.0 | 23.0 | 1446.0 |
| tree-collapsed | 100 | 2.0 | 1.0 | 164.0 |
| table-default | 10000 | 33.3 | 19.0 | 1128.0 |
| table-fill | 10000 | 10020.3 | 19.0 | 1128.0 |
| table-grid-6 | 10000 | 10036.0 | 18.0 | 4095.0 |
| table-rich-grid-6 | 10000 | 10036.0 | 18.0 | 6903.0 |
| tree-open | 10000 | 62.0 | 23.0 | 1452.0 |
| tree-collapsed | 10000 | 62.0 | 23.0 | 1807.0 |

All 137 per-workload primary-entity and App release checks passed. RSS includes sequential process history and owner data; it is not an ownership or leak report.

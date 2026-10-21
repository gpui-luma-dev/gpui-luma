# SDK headless baseline

Diagnostic CPU batch means on GPUI's test platform. These are not native frame-time percentiles.

| Scene | Look | Size | Primary controls | Warm redraw µs | Allocations/op | Requested bytes/op |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| forms | sdk | 10 | 70 | 1074.067 | 8229.0 | 4036012 |
| forms | sdk | 100 | 700 | 11290.850 | 82832.0 | 41006300 |
| forms | sdk | 1000 | 7000 | 177546.104 | 835285.0 | 420062452 |
| forms | shadcn | 10 | 70 | 1359.142 | 15194.0 | 4331981 |
| forms | shadcn | 100 | 700 | 14287.425 | 152436.0 | 43899497 |
| forms | shadcn | 1000 | 7000 | 208196.562 | 1531394.0 | 448906137 |
| forms | radix | 10 | 70 | 2247.300 | 48259.0 | 7492428 |
| forms | radix | 100 | 700 | 23790.275 | 483275.0 | 76218678 |
| forms | radix | 1000 | 7000 | 299503.604 | 4840187.0 | 772988754 |
| navigation | sdk | 100 | 7 | 2301.167 | 21196.0 | 9026403 |
| navigation | sdk | 10000 | 7 | 124134.354 | 765367.0 | 434443223 |
| collections | sdk | 100 | 2 | 377.333 | 2467.0 | 1319323 |
| collections | sdk | 10000 | 2 | 451.229 | 2479.0 | 1331611 |
| layout | sdk | 10 | 11 | 94.767 | 623.0 | 432629 |
| layout | sdk | 100 | 101 | 693.942 | 4529.0 | 3562117 |
| layout | sdk | 1000 | 1001 | 6478.812 | 43535.0 | 34739621 |
| color | sdk | 1 | 2 | 237.317 | 5536.0 | 1950712 |
| color | sdk | 10 | 20 | 2715.567 | 55233.0 | 19513516 |
| color | sdk | 100 | 200 | 23797.358 | 539344.0 | 189676364 |
| plot | sdk | 100 | 1 | 43.033 | 409.0 | 362655 |
| plot | sdk | 10000 | 1 | 925.833 | 455.0 | 10229175 |
| plot | sdk | 100000 | 1 | 2512.021 | 456.0 | 11885600 |

## Lifecycle observations

Tracked control weak references are checked after removal and two additional frames. RSS includes allocator and GPUI caches; growth alone does not establish a leak.

- forms/sdk/10: released 70 of 70 tracked control entities.
- forms/sdk/100: released 700 of 700 tracked control entities.
- forms/sdk/1000: released 7000 of 7000 tracked control entities.
- forms/shadcn/10: released 70 of 70 tracked control entities.
- forms/shadcn/100: released 700 of 700 tracked control entities.
- forms/shadcn/1000: released 7000 of 7000 tracked control entities.
- forms/radix/10: released 70 of 70 tracked control entities.
- forms/radix/100: released 700 of 700 tracked control entities.
- forms/radix/1000: released 7000 of 7000 tracked control entities.
- navigation/sdk/100: released 7 of 7 tracked control entities.
- navigation/sdk/10000: released 7 of 7 tracked control entities.
- collections/sdk/100: released 2 of 2 tracked control entities.
- collections/sdk/10000: released 2 of 2 tracked control entities.
- layout/sdk/10: released 11 of 11 tracked control entities.
- layout/sdk/100: released 101 of 101 tracked control entities.
- layout/sdk/1000: released 1001 of 1001 tracked control entities.
- color/sdk/1: released 2 of 2 tracked control entities.
- color/sdk/10: released 20 of 20 tracked control entities.
- color/sdk/100: released 200 of 200 tracked control entities.
- plot/sdk/100: released 1 of 1 tracked control entities.
- plot/sdk/10000: released 1 of 1 tracked control entities.
- plot/sdk/100000: released 1 of 1 tracked control entities.

See the JSONL data for cold construction, replacement/first paint, data updates, theme revision fanout, idle executor drain, memory checkpoints, and all batch samples.

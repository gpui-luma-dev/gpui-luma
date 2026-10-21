#!/usr/bin/env python3
"""Validate SDK perf JSONL and summarize diagnostic batch means, not frame percentiles."""
import argparse
import json
import math
from pathlib import Path


def load_records(path):
    records = [json.loads(line) for line in Path(path).read_text().splitlines() if line.strip()]
    if not records or records[0].get("type") != "environment" or records[0].get("schema_version") != 1:
        raise ValueError("expected schema version 1 environment record first")
    if records[-1].get("type") != "complete":
        raise ValueError("missing completion record; benchmark output is incomplete")
    measurements = set()
    for record in records[1:]:
        if record.get("type") in ("measurement", "infrastructure"):
            samples = record["batch_mean_ns"]
            if len(samples) != 5 or any(not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in samples):
                raise ValueError("expected five finite, nonnegative batch means")
            if record["iterations"] <= 0:
                raise ValueError("iterations must be positive")
            if not math.isclose(record["median_ns"], sorted(samples)[2], abs_tol=0.01):
                raise ValueError("median does not match batch means")
            for field in ("allocations_per_op", "requested_bytes_per_op"):
                if not math.isfinite(record[field]) or record[field] < 0:
                    raise ValueError(f"invalid {field}")
            key = (record.get("scene", "infrastructure"), record.get("look", "sdk"), record.get("size", 1), record.get("phase", record.get("workload")))
            if key in measurements:
                raise ValueError(f"duplicate measurement: {key}")
            measurements.add(key)
        elif record.get("type") == "release_check":
            if not 0 <= record["released"] <= record["tracked"]:
                raise ValueError("invalid release count")
        elif record.get("type") not in ("lifecycle_memory", "complete"):
            raise ValueError(f"unknown record type: {record.get('type')}")
    if not measurements:
        raise ValueError("no measurements")
    released_fixtures = sum(record.get("type") == "release_check" for record in records)
    if records[-1]["fixtures"] != released_fixtures:
        raise ValueError("completion count does not match fixture release checks")
    return records


def summarize(records):
    lines = ["# SDK headless baseline", "", "Diagnostic CPU batch means on GPUI's test platform. These are not native frame-time percentiles.", "",
             "| Scene | Look | Size | Primary controls | Warm redraw µs | Allocations/op | Requested bytes/op |", "| --- | --- | ---: | ---: | ---: | ---: | ---: |"]
    for row in records:
        if row.get("type") == "measurement" and row["phase"] == "warm_redraw":
            lines.append(f"| {row['scene']} | {row['look']} | {row['size']} | {row['controls']} | {row['median_ns'] / 1000:.3f} | {row['allocations_per_op']:.1f} | {row['requested_bytes_per_op']:.0f} |")
    lines += ["", "## Lifecycle observations", "", "Tracked control weak references are checked after removal and two additional frames. RSS includes allocator and GPUI caches; growth alone does not establish a leak.", ""]
    for row in records:
        if row.get("type") == "release_check":
            lines.append(f"- {row['scene']}/{row['look']}/{row['size']}: released {row['released']} of {row['tracked']} tracked control entities.")
    lines += ["", "See the JSONL data for cold construction, replacement/first paint, data updates, theme revision fanout, idle executor drain, memory checkpoints, and all batch samples.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = summarize(load_records(args.input))
    if args.output:
        args.output.write_text(result)
    else:
        print(result)


if __name__ == "__main__":
    main()

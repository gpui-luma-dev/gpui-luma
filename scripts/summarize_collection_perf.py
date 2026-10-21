#!/usr/bin/env python3
"""Validate and summarize the interaction-focused collection benchmark JSONL."""
import argparse
import json
import math
from pathlib import Path
import statistics

CASES = {"table-default", "table-fill", "table-grid-6", "table-rich-grid-6", "table-fixed-grid-6", "table-fixed-rich-grid-6", "tree-open", "tree-collapsed"}
BASE = {"construct_drop", "first_paint", "warm_redraw", "wheel", "keyboard", "selection", "replace", "single_value_replace", "sort", "filter", "jump"}


def expected_phases(case, size):
    phases = BASE.copy()
    if case.startswith("tree-"):
        phases.add("expand_collapse")
    if case in {"table-grid-6", "table-rich-grid-6", "table-fixed-grid-6", "table-fixed-rich-grid-6"}:
        phases.add("column_resize")
    if case == "tree-collapsed" and size < 202:
        phases -= {"keyboard", "selection"}
    if case == "tree-collapsed" and (size + 100) // 101 <= 22:
        phases.discard("wheel")
    return phases


def summarize(path):
    records = [json.loads(line) for line in Path(path).read_text().splitlines() if line.strip()]
    if len(records) < 3 or records[0].get("type") != "environment" or records[0].get("schema_version") != 2 or records[-1].get("type") != "complete":
        raise ValueError("missing schema-2 metadata or completion footer")
    env = records[0]
    cases, sizes = env["cases"], env["sizes"]
    if not cases or len(set(cases)) != len(cases) or not set(cases) <= CASES:
        raise ValueError("invalid case manifest")
    if not sizes or len(set(sizes)) != len(sizes) or any(type(size) is not int or not 100 <= size <= 100_000 for size in sizes):
        raise ValueError("invalid size manifest")
    expected = {(case, size, phase) for case in cases for size in sizes for phase in expected_phases(case, size)}
    rows = {}
    for item in records[1:-1]:
        if item.get("type") != "measurement" or item.get("released") is not True or item.get("app_released") is not True:
            raise ValueError("invalid measurement or failed release check")
        key = item["case"], item["size"], item["phase"]
        if key not in expected or key in rows:
            raise ValueError("unknown or duplicate workload")
        samples = item["batch_mean_ns"]
        if len(samples) != 5 or any(not isinstance(x, (float, int)) or not math.isfinite(x) or x < 0 for x in samples):
            raise ValueError("invalid batch means")
        if not math.isclose(statistics.median(samples), item["median_ns"], rel_tol=1e-9):
            raise ValueError("median differs from batches")
        for field in ["allocations_per_op", "requested_bytes_per_op", "presenter_calls_initial_setup", "presenter_calls_all_passes", "operation_calls_all_passes"]:
            if not isinstance(item[field], (int, float)) or not math.isfinite(item[field]) or item[field] < 0:
                raise ValueError("invalid operation or allocation counter")
        if item["operation_calls_all_passes"] <= 0:
            raise ValueError("missing operations")
        rows[key] = item
    if set(rows) != expected or records[-1].get("measurements") != len(rows):
        raise ValueError("incomplete workload matrix")
    columns = ["warm_redraw", "wheel", "keyboard", "replace", "single_value_replace", "sort", "filter", "jump", "expand_collapse", "column_resize"]
    lines = ["# Collection interaction measurements", "", "Headless release CPU milliseconds; medians of five batch means. Mock fonts/graphics; not native input latency.", "", "| Case | Loaded records/nodes | " + " | ".join(columns) + " |", "| --- | ---: | " + " | ".join(["---:"] * len(columns)) + " |"]
    failed_default_reveal = "table-default" in cases and env.get("table_default_distant_reveal_verified") is not True
    for size in sizes:
        for case in cases:
            values = [f"{rows[case,size,phase]['median_ns'] / 1e6:.3f}" if (case,size,phase) in rows else "—" for phase in columns]
            if case == "table-default" and failed_default_reveal:
                values[columns.index("jump")] += "*"
            lines.append(f"| {case} | {size} | " + " | ".join(values) + " |")
    if failed_default_reveal:
        lines += ["", "*Default Table distant reveal did not reach the unmeasured target in this capture. Its jump value measures an unsuccessful API request, not successful positioning latency; see collection-analysis.md.*"]
    lines += ["", "## Presenter work", "", "Counters include warmups, timed batches, and the separate allocation pass. Initial setup includes construction, first layout, resize, and focus; operation counts normalize all passes.", "", "| Case | Size | First-paint presenter calls/op | Warm presenter calls/op | Warm allocations/op |", "| --- | ---: | ---: | ---: | ---: |"]
    for size in sizes:
        for case in cases:
            cold, warm = rows[case,size,"first_paint"], rows[case,size,"warm_redraw"]
            lines.append(f"| {case} | {size} | {cold['presenter_calls_all_passes'] / cold['operation_calls_all_passes']:.1f} | {warm['presenter_calls_all_passes'] / warm['operation_calls_all_passes']:.1f} | {warm['allocations_per_op']:.1f} |")
    lines += ["", f"All {len(rows)} per-workload primary-entity and App release checks passed. RSS includes sequential process history and owner data; it is not an ownership or leak report.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = summarize(args.input)
    if args.output:
        args.output.write_text(result)
    else:
        print(result)


if __name__ == "__main__":
    main()

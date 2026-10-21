import json
import tempfile
import unittest
from pathlib import Path
from summarize_sdk_perf import load_records, summarize


class PerfOutputTests(unittest.TestCase):
    def fixture(self):
        return [{"type": "environment", "schema_version": 1},
                {"type": "measurement", "scene": "forms", "look": "sdk", "size": 10, "controls": 70,
                 "phase": "warm_redraw", "iterations": 5, "batch_mean_ns": [10, 30, 20, 40, 50],
                 "median_ns": 30, "allocations_per_op": 2, "requested_bytes_per_op": 40},
                {"type": "release_check", "tracked": 70, "released": 70, "scene": "forms", "look": "sdk", "size": 10},
                {"type": "complete", "fixtures": 1}]

    def load(self, records):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "results.jsonl"
            path.write_text("\n".join(json.dumps(record) for record in records))
            return load_records(path)

    def test_valid_output_preserves_scene_identity_and_units(self):
        output = summarize(self.load(self.fixture()))
        self.assertIn("| forms | sdk | 10 | 70 | 0.030 | 2.0 | 40 |", output)
        self.assertIn("not native frame-time percentiles", output)

    def test_rejects_wrong_median_and_duplicate_workload(self):
        records = self.fixture()
        records[1]["median_ns"] = 100
        with self.assertRaises(ValueError):
            self.load(records)
        records = self.fixture()
        records.insert(2, records[1].copy())
        with self.assertRaises(ValueError):
            self.load(records)

    def test_rejects_incomplete_or_nonfinite_samples(self):
        for samples in ([1], [1, 2, 3, 4, float("inf")]):
            records = self.fixture()
            records[1]["batch_mean_ns"] = samples
            with self.assertRaises(ValueError):
                self.load(records)

    def test_rejects_unknown_schema_and_empty_measurements(self):
        with self.assertRaises(ValueError):
            self.load([{"type": "environment", "schema_version": 9}])
        with self.assertRaises(ValueError):
            self.load([{"type": "environment", "schema_version": 1}])

    def test_rejects_partial_run_and_wrong_fixture_count(self):
        with self.assertRaises(ValueError):
            self.load(self.fixture()[:-1])
        records = self.fixture()
        records[-1]["fixtures"] = 2
        with self.assertRaises(ValueError):
            self.load(records)

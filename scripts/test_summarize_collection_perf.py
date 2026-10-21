import json
from pathlib import Path
import tempfile
import unittest
from summarize_collection_perf import expected_phases, summarize

class CollectionSummaryTests(unittest.TestCase):
    def records(self):
        records = [{"type":"environment", "schema_version":2, "cases":["table-default"], "sizes":[100]}]
        for phase in sorted(expected_phases("table-default",100)):
            records.append(dict(type="measurement", case="table-default", size=100, phase=phase, released=True, app_released=True, batch_mean_ns=[1,2,3,4,5], median_ns=3, allocations_per_op=1, requested_bytes_per_op=1, presenter_calls_initial_setup=2, presenter_calls_all_passes=2, operation_calls_all_passes=1))
        return records + [{"type":"complete", "measurements":len(records)-1}]
    def render(self,records):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/"results.jsonl"
            path.write_text("\n".join(json.dumps(r) for r in records))
            return summarize(path)
    def test_complete_report(self):
        self.assertIn("All 11",self.render(self.records()))
    def test_verified_distant_reveal_has_no_failure_annotation(self):
        data=self.records()
        self.assertIn("unsuccessful API request",self.render(data))
        data[0]["table_default_distant_reveal_verified"]=True
        self.assertNotIn("unsuccessful API request",self.render(data))
    def test_missing_phase_even_with_matching_footer(self):
        data=self.records();data.pop(1);data[-1]["measurements"]-=1
        with self.assertRaises(ValueError): self.render(data)
    def test_duplicate(self):
        data=self.records();data.insert(1,data[1]);data[-1]["measurements"]+=1
        with self.assertRaises(ValueError): self.render(data)
    def test_invalid_numeric_and_release(self):
        for field,value in [("allocations_per_op",float("nan")),("median_ns",42),("released",False),("app_released",False),("operation_calls_all_passes",0)]:
            data=self.records();data[1][field]=value
            with self.assertRaises(ValueError): self.render(data)
    def test_partial_output(self):
        with self.assertRaises(ValueError): self.render(self.records()[:-1])

if __name__ == "__main__": unittest.main()

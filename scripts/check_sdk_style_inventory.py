"""Validate the reconciled SDK style audit without launching an application."""
import csv
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWED = {
    "migrated", "superseded", "test fixture", "retained: mathematical",
    "retained: behavior", "retained: validation", "retained: interaction",
    "retained: layout", "retained: data", "retained: absence",
    "retained: derived calculation",
}


def main():
    with (ROOT / "docs/sdk-style-candidates.csv").open() as file:
        rows = list(csv.DictReader(file))
    for row in rows:
        assert row["status"] in ALLOWED, row
        assert row["rationale"].strip(), row
        assert row["owning field or classification"].strip(), row
        assert (ROOT / row["source"]).is_file(), row
        assert int(row["line"]) >= 0, row
    print(f"Validated {len(rows)} classified audit entries.")
    for status, count in sorted(Counter(row["status"] for row in rows).items()):
        print(f"{status}: {count}")


if __name__ == "__main__":
    main()

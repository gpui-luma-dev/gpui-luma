"""Select full native CI unless every changed file belongs to the docs runtime/site."""

import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DOCS_PREFIXES = ("docs/", "apps/docs-embed/")


def needs_native(paths):
    return not paths or any(not path.startswith(DOCS_PREFIXES) for path in paths)


def detect_native(base, root=ROOT):
    # Manual runs, new branches, and unavailable history retain full coverage.
    if not base or set(base) == {"0"}:
        return True
    try:
        result = subprocess.run(
            ["git", "diff", "--no-renames", "--name-only", "-z", base, "HEAD", "--"],
            cwd=root, check=True, capture_output=True,
        )
    except subprocess.CalledProcessError:
        return True
    # Disable rename detection so a Rust file moved into docs still runs native CI.
    return needs_native([os.fsdecode(path) for path in result.stdout.split(b"\0") if path])


if __name__ == "__main__":
    native = detect_native(os.environ.get("CI_BASE_SHA", ""))
    line = f"native={str(native).lower()}\n"
    print(line, end="")
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a") as stream:
            stream.write(line)

"""Offline CI routing regressions, including renames and missing history."""

from pathlib import Path
import subprocess
import tempfile
import unittest

from ci_scope import detect_native, needs_native


class ScopeTests(unittest.TestCase):
    def test_only_site_and_browser_runtime_changes_skip_native_suites(self):
        self.assertFalse(needs_native(["docs/button.html", "apps/docs-embed/src/web.rs"]))
        self.assertFalse(needs_native(["docs/examples/embed.js", "docs/site.css"]))

    def test_shared_rust_dependencies_scripts_and_workflows_keep_full_ci(self):
        for path in ["crates/sdk/src/lib.rs", "Cargo.lock", "apps/common/assets/font.ttf",
                     ".github/workflows/ci.yml", ".github/workflows/docs.yml", "scripts/ci_scope.py"]:
            with self.subTest(path=path):
                self.assertTrue(needs_native(["docs/button.html", path]))
        self.assertTrue(needs_native([]))

    def test_git_comparison_preserves_deleted_source_and_falls_back_for_missing_history(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def git(*args):
                return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL).decode().strip()

            git("init", "-q")
            source = root / "crates/sdk/src/lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn example() {}\n")
            git("add", ".")
            git("-c", "user.name=CI test", "-c", "user.email=ci@example.invalid", "commit", "-qm", "Source")
            base = git("rev-parse", "HEAD")
            (root / "docs").mkdir()
            source.rename(root / "docs/example.rs")
            git("add", "-A")
            git("-c", "user.name=CI test", "-c", "user.email=ci@example.invalid", "commit", "-qm", "Move into docs")
            self.assertTrue(detect_native(base, root))
            self.assertTrue(detect_native("missing-ref", root))
            self.assertTrue(detect_native("", root))
            self.assertTrue(detect_native("0" * 40, root))
            base = git("rev-parse", "HEAD")
            (root / "docs/example.rs").write_text("pub fn updated() {}\n")
            git("add", ".")
            git("-c", "user.name=CI test", "-c", "user.email=ci@example.invalid", "commit", "-qm", "Edit docs")
            self.assertFalse(detect_native(base, root))


if __name__ == "__main__":
    unittest.main()

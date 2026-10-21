"""Packaging regressions; runs with Python's standard library only."""

from pathlib import Path
import tempfile
import unittest

from package_runtime import ASSETS, publish, verify


class RuntimePackagingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.source = Path(self.temporary.name) / "source"
        self.destination = Path(self.temporary.name) / "deployment"
        self.source.mkdir()
        for name in ASSETS:
            (self.source / name).write_bytes(name.encode())

    def test_rebuild_keeps_only_current_matching_pair(self):
        first = publish(self.source, self.destination)
        self.assertEqual(publish(self.source, self.destination), first)
        (self.destination / "luma_docs_embed.js").write_text("obsolete unversioned loader")
        (self.source / ASSETS[1]).write_bytes(b"updated wasm")
        second = publish(self.source, self.destination)
        self.assertNotEqual(first, second)
        self.assertFalse((self.destination / first).exists())
        self.assertEqual({p.name for p in self.destination.iterdir()}, {second, "manifest.json"})
        for name in ASSETS:
            self.assertEqual((self.destination / second / name).read_bytes(), (self.source / name).read_bytes())
        verify(self.destination)

    def test_missing_or_empty_input_preserves_previous_deployment(self):
        version = publish(self.source, self.destination)
        (self.source / ASSETS[1]).unlink()
        with self.assertRaises(FileNotFoundError):
            publish(self.source, self.destination)
        verify(self.destination)
        (self.source / ASSETS[1]).touch()
        with self.assertRaises(ValueError):
            publish(self.source, self.destination)
        self.assertTrue((self.destination / version).is_dir())
        verify(self.destination)

    def test_verification_rejects_mismatched_assets_and_stale_versions(self):
        version = publish(self.source, self.destination)
        (self.destination / "stale").mkdir()
        with self.assertRaisesRegex(ValueError, "obsolete"):
            verify(self.destination)
        (self.destination / "stale").rmdir()
        (self.destination / version / ASSETS[0]).write_text("wrong loader")
        with self.assertRaisesRegex(ValueError, "match"):
            verify(self.destination)


if __name__ == "__main__":
    unittest.main()

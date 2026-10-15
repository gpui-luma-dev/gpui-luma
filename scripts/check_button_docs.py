"""Compile the Button checkpoint's HTML examples without launching an application."""

from html.parser import HTMLParser
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = {
    "button.html": [0, 1],
    "index.html": 1,  # Cargo.toml precedes the complete Rust application.
    "look-radix.html": 0,
    "look-shadcn.html": 0,
    "architecture.html": 0,
    "looks.html": 0,
}


class CodeBlocks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.blocks = []
        self.current = None

    def handle_starttag(self, tag, attrs):
        if tag == "pre":
            self.current = []

    def handle_data(self, data):
        if self.current is not None:
            self.current.append(data)

    def handle_endtag(self, tag):
        if tag == "pre" and self.current is not None:
            self.blocks.append("".join(self.current))
            self.current = None


def example_source():
    modules = ["#![allow(unused)]"]
    for page, selection in EXAMPLES.items():
        indices = selection if isinstance(selection, list) else [selection]
        parser = CodeBlocks()
        parser.feed((ROOT / "docs" / page).read_text())
        if any(index >= len(parser.blocks) for index in indices):
            raise ValueError(f"Missing code examples {indices} in {page}")
        code = "\n".join(parser.blocks[index] for index in indices)
        if page in {"architecture.html", "looks.html"}:
            code = (
                "struct Example { save_requests: usize }\n"
                "fn build(cx: &mut gpui::Context<Example>) {\n" + code + "\n}"
            )
        modules.append(f"mod {page.removesuffix('.html').replace('-', '_')} {{\n{code}\n}}")
    return "\n".join(modules)


def main():
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"], cwd=ROOT,
    ))
    dependencies = {
        dependency.get("rename") or dependency["name"]: dependency
        for package in metadata["packages"] for dependency in package["dependencies"]
    }
    with tempfile.TemporaryDirectory(prefix="luma-button-docs-") as directory:
        package = Path(directory)
        manifest = ['[package]', 'name = "luma-button-doc-check"', 'version = "0.0.0"',
                    'edition = "2024"', '[dependencies]']
        for name in ["gpui", "gpui_platform"]:
            spec = dependencies[name]
            fields = [f'package = {json.dumps(spec["name"])}',
                      f'version = {json.dumps(spec["req"])}',
                      f'features = {json.dumps(spec["features"])}']
            manifest.append(f'{name} = {{ {", ".join(fields)} }}')
        for name, path in [("gpui-luma", "sdk"), ("gpui-luma-look-radix", "look-radix"),
                           ("gpui-luma-look-shadcn", "look-shadcn")]:
            manifest.append(f'{name} = {{ path = {json.dumps(str(ROOT / "crates" / path))} }}')
        (package / "Cargo.toml").write_text("\n".join(manifest) + "\n")
        (package / "src").mkdir()
        (package / "src/lib.rs").write_text(example_source())
        # Retain the workspace's dependency resolutions. Cargo adds only this
        # temporary package; no repository manifests or lockfiles are modified.
        shutil.copyfile(ROOT / "Cargo.lock", package / "Cargo.lock")
        subprocess.run(["cargo", "check", "--manifest-path", str(package / "Cargo.toml"),
                        "--target-dir", str(ROOT / "target")], check=True, cwd=ROOT)
    count = sum(len(value) if isinstance(value, list) else 1 for value in EXAMPLES.values())
    print(f"Compiled {count} Button documentation examples.")


if __name__ == "__main__":
    main()

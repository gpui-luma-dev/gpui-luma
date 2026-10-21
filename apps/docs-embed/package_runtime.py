"""Publish and validate the single content-versioned runtime shipped with the docs."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil

ASSETS = ("luma_docs_embed.js", "luma_docs_embed_bg.wasm")


def read_assets(directory):
    assets = {name: (directory / name).read_bytes() for name in ASSETS}
    if not all(assets.values()):
        raise ValueError("Runtime assets must not be empty")
    return assets


def version_for(assets):
    digest = hashlib.sha256()
    for name in ASSETS:
        digest.update(name.encode() + b"\0" + assets[name])
    return digest.hexdigest()[:20]


def verify(destination):
    version = json.loads((destination / "manifest.json").read_text())["version"]
    if not isinstance(version, str) or len(version) != 20 or any(c not in "0123456789abcdef" for c in version):
        raise ValueError("Invalid runtime version")
    if version_for(read_assets(destination / version)) != version:
        raise ValueError("Runtime assets do not match the manifest")
    if {p.name for p in destination.iterdir()} != {version, "manifest.json"}:
        raise ValueError("Deployment contains obsolete runtime assets")
    if {p.name for p in (destination / version).iterdir()} != set(ASSETS):
        raise ValueError("Unexpected files in the runtime version directory")


def publish(source, destination):
    # Validate inputs before touching an existing preview build.
    assets = read_assets(source)
    version = version_for(assets)
    current = destination / version
    current.mkdir(parents=True, exist_ok=True)
    for name, data in assets.items():
        (current / name).write_bytes(data)
    manifest = destination / "manifest.json"
    temporary = manifest.with_suffix(".tmp")
    temporary.write_text(json.dumps({"version": version}) + "\n")
    temporary.replace(manifest)
    # Only generated pkg contents are removed; sources and fonts live elsewhere.
    for path in destination.iterdir():
        if path.name not in {version, "manifest.json"}:
            if path.is_dir() and not path.is_symlink():
                shutil.rmtree(path)
            else:
                path.unlink()
    verify(destination)
    return version


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Validate the deployment without changing it")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    destination = root / "docs/examples/pkg"
    if args.check:
        verify(destination)
    else:
        publish(root / "apps/docs-embed/pkg", destination)

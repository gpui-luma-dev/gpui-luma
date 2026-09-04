#!/usr/bin/env bash

set -euo pipefail

crate_root="$(cd "$(dirname "$0")/.." && pwd)"
exec cargo run --manifest-path "$crate_root/Cargo.toml" --features font-download --bin download-google-fonts -- "$@"

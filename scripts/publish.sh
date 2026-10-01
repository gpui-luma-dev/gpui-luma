#!/bin/bash
# Local crates.io release, compatible with macOS Bash 3.2.
set +x
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/publish.sh [--publish] [--keychain|--env] [--allow-dirty]

Default: check that all four manifest versions are unused, then run a dry run.
--publish: after that dry run, upload all four packages and download each
published archive to verify it matches the local archive byte for byte.

Authentication defaults to Cargo's native macOS Keychain provider.
--env uses CARGO_REGISTRY_TOKEN or CARGO_REGISTRIES_CRATES_IO_TOKEN instead.
--allow-dirty explicitly permits packaging uncommitted changes.

Requires macOS, current stable Cargo, Git, curl, cmp, and network access.
Stops if any selected version already exists; never overwrites a release.
If an upload partially succeeds, inspect crates.io before attempting recovery.
Does not change versions, create Git tags, run GUI apps, or configure CI.

Store a token separately, pasting it at Cargo's prompt:
  cargo login --registry crates-io \
    --config 'registries.crates-io.credential-provider="cargo:macos-keychain"'
EOF
}

publish=false
allow_dirty=false
provider="cargo:macos-keychain"
options=()
for argument in "$@"; do
    case "$argument" in
        --publish) publish=true ;;
        --keychain) provider="cargo:macos-keychain" ;;
        --env) provider="cargo:token" ;;
        --allow-dirty) allow_dirty=true; options+=(--allow-dirty) ;;
        -h|--help) usage; exit 0 ;;
        *) printf 'Unsupported option.\n' >&2; usage >&2; exit 2 ;;
    esac
done

if [[ "$(uname -s)" != Darwin ]]; then
    printf 'This script requires macOS.\n' >&2
    exit 1
fi
for command in cargo git curl cmp cp mktemp; do
    if ! command -v "$command" >/dev/null 2>&1; then
        printf 'Required command is unavailable: %s\n' "$command" >&2
        exit 1
    fi
done
if [[ "$provider" == "cargo:token" &&
      -z "${CARGO_REGISTRY_TOKEN:-}" &&
      -z "${CARGO_REGISTRIES_CRATES_IO_TOKEN:-}" ]]; then
    printf 'Set CARGO_REGISTRY_TOKEN or CARGO_REGISTRIES_CRATES_IO_TOKEN for --env.\n' >&2
    exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/.." && pwd)"
cd -- "$repo_dir"
if [[ "$allow_dirty" == false && -n "$(git status --porcelain)" ]]; then
    printf 'Commit your changes first, or explicitly use --allow-dirty.\n' >&2
    exit 1
fi

version=""
packages=(gpui-luma gpui-luma-color gpui-luma-look-radix gpui-luma-look-shadcn)
selection=()
for package in "${packages[@]}"; do
    package_id="$(cargo pkgid --offline --locked --package "$package")"
    package_version="${package_id##*@}"
    if [[ -z "$version" ]]; then version="$package_version"; fi
    if [[ "$package_id" != *@* || "$package_version" != "$version" ]]; then
        printf 'All release packages must share a version; %s differs from %s.\n' "$package" "$version" >&2
        exit 1
    fi
    selection+=(--package "$package")
done

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/luma-publish.XXXXXX")"
trap 'rm -rf -- "$temp_dir"' EXIT
agent="gpui-luma-publish/$version (https://gpui-luma.dev)"
for package in "${packages[@]}"; do
    status="$(curl --silent --show-error --location --connect-timeout 15 --max-time 60 \
        --user-agent "$agent" --output "$temp_dir/response" --write-out '%{http_code}' \
        "https://crates.io/api/v1/crates/$package/$version")"
    case "$status" in
        404) printf 'Available version: %s %s\n' "$package" "$version" ;;
        200)
            printf '%s %s already exists. Stopping; nothing will be uploaded.\n' "$package" "$version" >&2
            exit 1 ;;
        *) printf 'Registry check failed for %s (HTTP %s).\n' "$package" "$status" >&2; exit 1 ;;
    esac
done

# A private target directory retains exactly the archives built for this run,
# regardless of the caller's CARGO_TARGET_DIR or other concurrent Cargo runs.
target_dir="$temp_dir/target"
common=(--registry crates-io --locked --manifest-path "$repo_dir/Cargo.toml"
    --target-dir "$target_dir"
    --config "registries.crates-io.credential-provider=\"$provider\"")
printf 'Running package/build verification; no upload yet.\n'
cargo publish --dry-run "${common[@]}" "${selection[@]}" ${options[@]+"${options[@]}"}
if [[ "$publish" == false ]]; then
    printf 'Dry run passed. To upload, rerun with --publish.\n'
    exit 0
fi

# Preserve the verified archives before Cargo's upload/cleanup phase.
for package in "${packages[@]}"; do
    archive="$package-$version.crate"
    archive_path="$target_dir/package/$archive"
    # Multi-package publish stages its archives in a temporary registry.
    if [[ ! -f "$archive_path" ]]; then
        archive_path="$target_dir/package/tmp-registry/$archive"
    fi
    if [[ ! -f "$archive_path" ]]; then
        printf 'Verified archive is missing: %s. Nothing will be uploaded.\n' "$archive" >&2
        exit 1
    fi
    cp "$archive_path" "$temp_dir/expected-$archive"
done

printf 'Publishing the four %s packages to crates.io.\n' "$version"
if ! cargo publish "${common[@]}" "${selection[@]}" ${options[@]+"${options[@]}"}; then
    printf 'Publish failed; some packages may already be published. Inspect crates.io before recovery.\n' >&2
    exit 1
fi

for package in "${packages[@]}"; do
    archive="$package-$version.crate"
    curl --fail --silent --show-error --location --retry 5 --retry-delay 2 \
        --connect-timeout 15 --max-time 60 --user-agent "$agent" \
        --output "$temp_dir/$archive" \
        "https://static.crates.io/crates/$package/$archive"
    if ! cmp -s "$temp_dir/expected-$archive" "$temp_dir/$archive"; then
        printf 'Published archive verification failed for %s. The release exists; inspect it before proceeding.\n' "$package" >&2
        exit 1
    fi
    printf 'Verified published contents: https://crates.io/crates/%s/%s\n' "$package" "$version"
done
printf 'All four %s releases published and verified. Project version remains %s.\n' "$version" "$version"

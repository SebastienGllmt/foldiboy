#!/usr/bin/env bash
set -euo pipefail

project_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -L)
test_root="${FOLDIBOY_TEST_ROOT:-$(dirname -- "$project_dir")/roms/tests}"

for suite in gb-test-roms SameSuite; do
    if [[ ! -e "$test_root/$suite/.git" ]]; then
        printf 'Missing checkout: %s/%s\nSee machine-docs/testing.md for upstream URLs.\n' "$test_root" "$suite" >&2
        exit 1
    fi
done

if command -v rgbasm >/dev/null 2>&1; then
    make -C "$test_root/SameSuite"
else
    # A project-scoped, pinned tool; does not change the machine's profiles.
    nix shell github:NixOS/nixpkgs/c8f90650c15282fa8656a041bfbbd2403997a9a7#rgbds \
        --command make -C "$test_root/SameSuite"
fi

cd -- "$project_dir/lib"
# Cargo's build script restores the locked WIT packages, including local audio.
cargo test --locked --workspace --no-run

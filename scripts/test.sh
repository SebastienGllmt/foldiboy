#!/usr/bin/env bash
set -euo pipefail

# Keep the logical path: the workspace may contain symlinks to canonical clones.
project_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -L)
export FOLDIBOY_TEST_ROOT="${FOLDIBOY_TEST_ROOT:-$(dirname -- "$project_dir")/roms/tests}"

for rom in \
    "gb-test-roms/cpu_instrs/individual/01-special.gb" \
    "gb-test-roms/dmg_sound/rom_singles/08-len ctr during power.gb" \
    "SameSuite/interrupt/ei_delay_halt.gb"; do
    if [[ ! -f "$FOLDIBOY_TEST_ROOT/$rom" ]]; then
        printf 'Missing test ROM: %s/%s\nRun scripts/setup-tests.sh or set FOLDIBOY_TEST_ROOT.\n' "$FOLDIBOY_TEST_ROOT" "$rom" >&2
        exit 1
    fi
done

cd -- "$project_dir/lib"
exec cargo test --locked --workspace --no-fail-fast "$@"

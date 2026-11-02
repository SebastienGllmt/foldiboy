#!/usr/bin/env bash
# Upstream mooneye acceptance suite adapter only.
# Keep project-authored tests and proof workflows separate; see README.md.
# Builds the mooneye acceptance ROMs that need only the CPU, RAM, IE and IF,
# and runs each on the definition. Run `run-singlestep-tests.sh` first: it emits the
# definition IR and builds the harness this script uses.
# Arguments are extra ROM names (relative to the suite's acceptance/ directory).
# The report, results/mooneye.json, records each ROM's verdict as the suite prescribes it.
set -euo pipefail
cd "$(dirname "$0")"
project_dir=$(cd ../.. && pwd -L)
FOLDIBOY_TEST_ROOT=${FOLDIBOY_TEST_ROOT:-$(dirname -- "$project_dir")/roms/tests}
MOONEYE=${MOONEYE:-$FOLDIBOY_TEST_ROOT/mooneye-test-suite}
Z3_LIB=${Z3_LIB:-/nix/store/bncr69nhmgvphbynb1a3g4a2jb9b8vj0-z3-4.12.5-lib/lib}
STEPS=${STEPS:-20000}
export LD_LIBRARY_PATH="$Z3_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
ROMS=(ei_sequence ei_timing rapid_di_ei reti_intr_timing interrupts/ie_push if_ie_registers "$@")
BUILD="$PWD/results/mooneye"
targets=()
for rom in "${ROMS[@]}"; do targets+=("$BUILD/acceptance/$rom.gb"); done
(cd "$MOONEYE" && nix-shell -p wla-dx gnumake --run "make --silent BUILD_PATH=$BUILD ${targets[*]}") >/dev/null
vectors/target/release/sm83-rom --definition-ir results/emitted/definition-ir.json --steps "$STEPS" --report results/mooneye.json "${targets[@]}"

#!/usr/bin/env bash
# Upstream mooneye acceptance suite adapter only.
# Keep project-authored tests and proof workflows separate; see README.md.
# Builds the mooneye acceptance ROMs that need only the CPU, RAM, IE and IF, and runs the cases of
# cases/mooneye.toml (each ROM with the boot state, expecting mooneye's pass registers) with
# krs cases. Arguments are passed to krs cases, e.g. `--name ei_sequence`.
# The report, results/mooneye.json, holds krs cases' record of each case (krs-cases/2).
set -euo pipefail
cd "$(dirname "$0")"
project_dir=$(cd ../.. && pwd -L)
FOLDIBOY_TEST_ROOT=${FOLDIBOY_TEST_ROOT:-$(dirname -- "$project_dir")/roms/tests}
MOONEYE=${MOONEYE:-$FOLDIBOY_TEST_ROOT/mooneye-test-suite}
KRS_ROOT=${KRS_ROOT:-/artifacts/worktrees/k-rust-foldiboy-pin}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
ROMS=(ei_sequence ei_timing rapid_di_ei reti_intr_timing interrupts/ie_push if_ie_registers)
BUILD="$PWD/results/mooneye"
targets=()
for rom in "${ROMS[@]}"; do targets+=("$BUILD/acceptance/$rom.gb"); done
(cd "$MOONEYE" && nix-shell -p wla-dx gnumake --run "make --silent BUILD_PATH=$BUILD ${targets[*]}") >/dev/null
report=$PWD/results/mooneye.json
cases=$PWD/cases/mooneye.toml
# The cases' expectations name the views package as their observer, which must be a package of the
# Cargo workspace krs reads, so krs runs from there.
cd ../sm83-views
status=0
CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" cases --manifest-path Cargo.toml --package foldiboy-sm83 --definition main \
  --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events \
  --output json "$cases" "$@" >"$report" || status=$?
python3 - "$report" <<'EOF'
import json, sys
for line in open(sys.argv[1]):
    record = json.loads(line)
    if "case" in record:
        detail = ""
        if record.get("difference"):
            difference = record["difference"]
            detail = f" at {difference.get('path')}: expected {difference['left']['value']['document']}, actual {difference['right']['value']['document']}"
        print(f"{record['case'].split('#')[-1]}: {record['verdict']}{detail}")
    elif "summary" in record:
        print(record["summary"])
EOF
exit "$status"

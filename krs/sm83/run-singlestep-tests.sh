#!/usr/bin/env bash
# Upstream SingleStepTests SM83 suite adapter only.
# Keep project-authored tests and proof workflows separate; see README.md.
# Checks the definition, converts the suite's vectors into case documents (singlestep-cases.py), and
# runs them with krs cases, which checks each vector's final state and bus cycles.
# Arguments are suite file names (default: all 500), e.g. `00 "cb 40"`.
# WZ=W,Z sets the initial unobservable W and Z (default 0,0); every vector must give the same verdict
# under any value. JOBS sets the workers (default 16; each holds a copy of the definition).
# Writes results/singlestep.jsonl (krs cases' records, krs-cases/2) and results/vectors.json
# (per suite file: vectors, met, and the first three failures).
set -euo pipefail
cd "$(dirname "$0")"
# Keep the logical path: the workspace may contain symlinks to canonical clones.
project_dir=$(cd ../.. && pwd -L)
FOLDIBOY_TEST_ROOT=${FOLDIBOY_TEST_ROOT:-$(dirname -- "$project_dir")/roms/tests}
KRS_ROOT=${KRS_ROOT:-/artifacts/worktrees/k-rust-foldiboy-pin}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
CORPUS=${CORPUS:-$FOLDIBOY_TEST_ROOT/sm83/v1}
JOBS=${JOBS:-16}
WZ=${WZ:-0,0}
krs=(--manifest-path Cargo.toml --package foldiboy-sm83 --definition main --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events)
CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" check "${krs[@]}"
cases=results/singlestep-cases
rm -rf "$cases"
python3 singlestep-cases.py --wz "$WZ" "$CORPUS" "$cases" "$@"
status=0
CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" cases "${krs[@]}" --jobs "$JOBS" --output json "$cases"/*.json \
  >results/singlestep.jsonl || status=$?
python3 - results/singlestep.jsonl results/vectors.json "$WZ" <<'EOF'
import json, sys
from collections import defaultdict
records, out, wz = sys.argv[1], sys.argv[2], sys.argv[3]
files = defaultdict(lambda: {"total": 0, "met": 0, "failures": []})
for line in open(records):
    record = json.loads(line)
    if "case" not in record:
        continue
    document, _, name = record["case"].rpartition("#")
    entry = files[document.rsplit("/", 1)[-1].removesuffix(".json")]
    entry["total"] += 1
    if record["verdict"] == "met":
        entry["met"] += 1
    elif len(entry["failures"]) < 3:
        failure = {"name": name, "verdict": record["verdict"]}
        if "difference" in record:
            difference = record["difference"]
            failure["at"] = difference.get("path") or difference.get("at")
            failure["expected"] = ((difference.get("left") or {}).get("value") or {}).get("document")
            failure["actual"] = ((difference.get("right") or {}).get("value") or {}).get("document")
        if "reason" in record:
            failure["reason"] = record["reason"]
        entry["failures"].append(failure)
total = sum(entry["total"] for entry in files.values())
met = sum(entry["met"] for entry in files.values())
report = {"total": total, "met": met, "wz": wz, "files": [{"file": name, **entry} for name, entry in sorted(files.items())]}
open(out, "w").write(json.dumps(report, indent=2) + "\n")
for name, entry in sorted(files.items()):
    if entry["met"] != entry["total"]:
        print(f"{name}: {entry['met']}/{entry['total']} met; first: {json.dumps(entry['failures'][0])}")
print(f"met {met}/{total}")
EOF
exit "$status"

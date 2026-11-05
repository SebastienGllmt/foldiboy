#!/usr/bin/env bash
# Runs the corpus on the Rust emulator's CPU and compares each run's end with the SM83 definition's
# run of the same case, with `krs compare`. Arguments are case ids (default: every case).
#
# - build.sh assembles the cases' ROMs.
# - sm83-rust-cpu (krs/sm83-rust-cpu) runs the Rust CPU from each case's initial state and writes
#   its end as a krs end document, build/ends/rust-cpu/<id>.json. Any other system is compared
#   the same way: put its end documents in build/ends/<system>/.
# - krs compare runs the case on the definition and compares the two ends on every cell, the bus
#   log under each view below (krs/sm83-views). The Rust CPU executes an instruction's accesses
#   at once, without internal cycles, so its bus is compared on the writes (required equal), and
#   on the reads and writes in order (reported: every case that dispatches an interrupt differs by
#   one read per dispatch, the discarded opcode read the definition's dispatch starts with).
#
# Writes results/corpus.json: per case and system, the end and each view's verdict.
set -euo pipefail
cd "$(dirname "$0")"
corpus=$PWD
KRS_ROOT=${KRS_ROOT:-/artifacts/worktrees/k-rust-foldiboy-pin}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
JOBS=${JOBS:-4}
VIEWS=(WritesView AccessesView)
REQUIRED=WritesView
./build.sh "$@"
if [[ $# -gt 0 ]]; then ids=("$@"); else mapfile -t ids < <(ls cases); fi
paths=()
for id in "${ids[@]}"; do paths+=("cases/$id/case.toml"); done

sm83=../krs/sm83
(cd "$sm83" && CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" emit-k --ir --manifest-path Cargo.toml --package foldiboy-sm83 \
  --definition main --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events \
  --output "$corpus/build/ir/sm83" >/dev/null)
(cd ../krs/sm83-rust-cpu && cargo build --release --offline --quiet)
rm -rf build/ends/rust-cpu build/compare
mkdir -p build/ends
../krs/sm83-rust-cpu/target/release/sm83-rust-cpu --definition-ir build/ir/sm83/definition-ir.json \
  --out build/ends/rust-cpu "${paths[@]}" | tee build/ends/rust-cpu.log

# One comparison per system, case and view, run from the views package: the observer must be a
# package of the Cargo workspace krs reads.
compare() {
  local system=$1 id=$2 view=$3
  local out="$corpus/build/compare/$system/$view/$id.json"
  mkdir -p "$(dirname "$out")"
  (cd "$corpus/../krs/sm83-views" && CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" compare --manifest-path Cargo.toml \
    --package foldiboy-sm83 --definition main --dependency foldiboy-sm83-core:core \
    --dependency foldiboy-sm83-events:events --observer foldiboy-sm83-views:views \
    "$corpus/cases/$id/case.toml" "$corpus/build/ends/$system/$id.json" --view "bus=$view" --output json \
    >"$out" 2>"$out.stderr") || true
}
export -f compare
export corpus KRS_BIN
# The first comparison prepares the definition and the observer; the others reuse them.
compare rust-cpu "${ids[0]}" "$REQUIRED"
for system in $(ls build/ends); do
  [[ -d build/ends/$system ]] || continue
  for id in "${ids[@]}"; do
    for view in "${VIEWS[@]}"; do
      [[ $system == rust-cpu && $id == "${ids[0]}" && $view == "$REQUIRED" ]] && continue
      echo "$system $id $view"
    done
  done
done | xargs -P "$JOBS" -L 1 bash -c 'compare "$0" "$1" "$2"'

mkdir -p results
python3 - "$REQUIRED" "${VIEWS[@]}" <<'EOF'
import json, sys
from pathlib import Path
required, views = sys.argv[1], sys.argv[2:]
ends = Path("build/ends")
cases, failed = {}, []
for system in sorted(p.name for p in ends.iterdir() if p.is_dir()):
    for end in sorted((ends / system).glob("*.json")):
        case = cases.setdefault(end.stem, {"case": end.stem, "systems": {}})
        record = {"halt": json.loads(end.read_text())["halt"], "views": {}}
        for view in views:
            path = Path("build/compare") / system / view / f"{end.stem}.json"
            text = path.read_text() if path.exists() else ""
            if not text.strip():
                stderr = Path(f"{path}.stderr")
                verdict = {"verdict": "error", "reason": stderr.read_text().strip() if stderr.exists() else "no output"}
            else:
                document = json.loads(text)
                verdict = {"verdict": document["verdict"]}
                if "difference" in document:
                    difference = document["difference"]
                    verdict["at"] = difference.get("path")
                    verdict["definition"] = ((difference.get("left") or {}).get("value") or {}).get("document")
                    verdict[system] = ((difference.get("right") or {}).get("value") or {}).get("document")
            record["views"][view] = verdict
            if view == required and verdict["verdict"] != "equal":
                failed.append(f"{system} {end.stem}")
        case["systems"][system] = record
log = Path("build/ends/rust-cpu.log")
for line in log.read_text().splitlines() if log.exists() else []:
    id, _, run = line.partition(": ")
    if id in cases and "rust-cpu" in cases[id]["systems"]:
        cases[id]["systems"]["rust-cpu"]["run"] = run
report = {"schema": "foldiboy-corpus/3", "required": required, "views": views, "cases": list(cases.values())}
Path("results/corpus.json").write_text(json.dumps(report, indent=2) + "\n")
for case in cases.values():
    for system, record in case["systems"].items():
        print(case["case"], system, " ".join(f"{view}={record['views'][view]['verdict']}" for view in views))
if failed:
    sys.exit(f"not equal under {required}: {', '.join(failed)}")
EOF

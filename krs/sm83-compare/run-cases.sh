#!/usr/bin/env bash
# Runs every comparator case in cases/ and checks its verdict against cases/expected.toml.
set -euo pipefail
cd "$(dirname "$0")"
KRS_ROOT=${KRS_ROOT:-/artifacts/worktrees/k-rust-foldiboy-pin}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
Z3_LIB=${Z3_LIB:-/nix/store/bncr69nhmgvphbynb1a3g4a2jb9b8vj0-z3-4.12.5-lib/lib}
export LD_LIBRARY_PATH="$Z3_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LIBRARY_PATH="$Z3_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
base=(--manifest-path Cargo.toml --package foldiboy-sm83-compare --definition compare
      --dependency foldiboy-sm83-observe:observe --dependency foldiboy-sm83-events:events)
CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" check "${base[@]}"
failed=0
for case in cases/*.json; do
  name=$(basename "$case" .json)
  expected=$(python3 -c 'import sys, tomllib; print(tomllib.load(open("cases/expected.toml", "rb"))[sys.argv[1]])' "$name")
  verdict=$(CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" run "${base[@]}" --case "$case" 2>&1 \
    | awk '/<k>/{f=1;next} /<\/k>/{f=0} f' | tr -s ' \n' ' ' | sed 's/^ //')
  if [[ $verdict == "$expected"* ]]; then
    echo "ok   $name"
  else
    echo "FAIL $name: $verdict"
    failed=1
  fi
done
exit "$failed"

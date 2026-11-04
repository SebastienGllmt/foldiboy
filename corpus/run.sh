#!/usr/bin/env bash
# Builds the cases, emits the SM83 and comparator definitions' IR, builds the runner, and runs the
# corpus on every system corpus.toml names. Arguments are case ids (default: every case).
# Writes results/corpus.json and each run's observation under build/observations/<system>/<case>.json.
set -euo pipefail
cd "$(dirname "$0")"
corpus=$PWD
KRS_ROOT=${KRS_ROOT:-/artifacts/worktrees/k-rust-foldiboy-pin}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
Z3_LIB=${Z3_LIB:-/nix/store/bncr69nhmgvphbynb1a3g4a2jb9b8vj0-z3-4.12.5-lib/lib}
export LD_LIBRARY_PATH="$Z3_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LIBRARY_PATH="$Z3_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
./build.sh "$@"
sm83=../krs/sm83
(cd "$sm83" && CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" emit-k --manifest-path Cargo.toml --package foldiboy-sm83 \
  --definition main --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events \
  --output "$corpus/build/ir/sm83" >/dev/null)
(cd ../krs/sm83-compare && CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" emit-k --manifest-path Cargo.toml \
  --package foldiboy-sm83-compare --definition compare --dependency foldiboy-sm83-observe:observe \
  --dependency foldiboy-sm83-events:events --output "$corpus/build/ir/compare" >/dev/null)
(cd "$sm83/vectors" && cargo build --release --offline --quiet --bin sm83-corpus)
mkdir -p results
"$sm83/vectors/target/release/sm83-corpus" --corpus corpus.toml \
  --definition-ir build/ir/sm83/definition-ir.json --compare-ir build/ir/compare/definition-ir.json \
  --observations build/observations --report results/corpus.json "$@"

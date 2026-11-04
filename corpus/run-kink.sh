#!/usr/bin/env bash
# Builds the cases and runs `krs kink VERB` over this corpus for the SM83 definition (FLOW.toml,
# mutations/*.toml). VERB and its options follow, e.g.:
#   corpus/run-kink.sh check                  # admit the definition and the cases, resolve every mutation
#   corpus/run-kink.sh results --output verdicts
#   corpus/run-kink.sh run --full             # every case that kills each mutation
# krs kink keeps definition.k equal to the definition's emission (it writes it when stale;
# `emit --check` fails instead). Prints the verb's JSON document on stdout.
set -euo pipefail
cd "$(dirname "$0")"
KRS_ROOT=${KRS_ROOT:-/artifacts/worktrees/k-rust-foldiboy-pin}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
Z3_LIB=${Z3_LIB:-/nix/store/bncr69nhmgvphbynb1a3g4a2jb9b8vj0-z3-4.12.5-lib/lib}
export LD_LIBRARY_PATH="$Z3_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
./build.sh >&2
cd ../krs/sm83
CARGO_TARGET_DIR="$PWD/target" exec "$KRS_BIN" kink "$@" --manifest-path Cargo.toml --package foldiboy-sm83 \
  --definition main --dependency foldiboy-sm83-core:core --dependency foldiboy-sm83-events:events --corpus ../../corpus

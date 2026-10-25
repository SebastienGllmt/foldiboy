#!/usr/bin/env bash
# Checks the definition, emits its IR, and runs it against the SingleStepTests corpus.
# Arguments are passed to the harness, e.g. `--files 00,40 --limit 100`.
set -euo pipefail
cd "$(dirname "$0")"
# Keep the logical path: the workspace may contain symlinks to canonical clones.
project_dir=$(cd ../.. && pwd -L)
FOLDIBOY_TEST_ROOT=${FOLDIBOY_TEST_ROOT:-$(dirname -- "$project_dir")/roms/tests}
KRS_ROOT=${KRS_ROOT:-/home/sebastiengllmt/git/workspaces/starstream-fv/inspirations/midnightntwrk/k-rust}
KRS_BIN=${KRS_BIN:-$KRS_ROOT/target/debug/krs}
CORPUS=${CORPUS:-$FOLDIBOY_TEST_ROOT/sm83/v1}
Z3_LIB=${Z3_LIB:-/nix/store/bncr69nhmgvphbynb1a3g4a2jb9b8vj0-z3-4.12.5-lib/lib}
export LD_LIBRARY_PATH="$Z3_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LIBRARY_PATH="$Z3_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" check --manifest-path Cargo.toml --package foldiboy-sm83 --definition main
CARGO_TARGET_DIR="$PWD/target" "$KRS_BIN" emit-k --manifest-path Cargo.toml --package foldiboy-sm83 --definition main --output results/emitted >/dev/null
(cd vectors && cargo build --release --offline --quiet)
vectors/target/release/sm83-vectors --definition-ir results/emitted/definition-ir.json --corpus "$CORPUS" --report results/vectors.json "$@"

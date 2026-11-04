#!/usr/bin/env bash
# Builds every case under cases/ (or the ids given) to build/<id>.gb, with its symbol and map files,
# using RGBDS from nixpkgs (draft-personal/corpus-contract D-ASSEMBLER). The ROM is what every system runs.
# Every byte the case does not place is $DD, the lock opcode, so a jump outside the case's code ends
# the run Locked at once: a wrong jump target is a visible end, not a run that never stops.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p build
if [[ $# -gt 0 ]]; then ids=("$@"); else mapfile -t ids < <(ls cases); fi
nix-shell -p rgbds --run "set -euo pipefail
for id in ${ids[*]}; do
  rgbasm -Wall -I include -o build/\$id.o cases/\$id/case.asm
  rgblink -p 0xDD -o build/\$id.gb -n build/\$id.sym -m build/\$id.map build/\$id.o
  rgbfix -v -p 0xDD build/\$id.gb
done"
echo "built ${#ids[@]} cases in build/"

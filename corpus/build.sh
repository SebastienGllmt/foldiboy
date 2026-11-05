#!/usr/bin/env bash
# Builds every case under cases/ (or the ids given) to cases/<id>/case.gb, with its symbol and map
# files, using RGBDS from nixpkgs (draft-personal/corpus-contract D-ASSEMBLER). The ROM is what every system runs.
# Every byte the case does not place is $DD, the lock opcode, so a jump outside the case's code ends
# the run Locked at once: a wrong jump target is a visible end, not a run that never stops.
set -euo pipefail
cd "$(dirname "$0")"
if [[ $# -gt 0 ]]; then ids=("$@"); else mapfile -t ids < <(ls cases); fi
nix-shell -p rgbds --run "set -euo pipefail
for id in ${ids[*]}; do
  rgbasm -Wall -I include -o cases/\$id/case.o cases/\$id/case.asm
  rgblink -p 0xDD -o cases/\$id/case.gb -n cases/\$id/case.sym -m cases/\$id/case.map cases/\$id/case.o
  rgbfix -v -p 0xDD cases/\$id/case.gb
  rm cases/\$id/case.o
done"
echo "built ${#ids[@]} cases"

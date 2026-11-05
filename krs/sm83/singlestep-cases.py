#!/usr/bin/env python3
"""Converts the SingleStepTests SM83 suite (v1) into krs case documents.

Each vector is one instruction from a boundary state. It becomes a named case whose inputs are
the vector's initial state and whose expectation is the suite's final state and bus cycles, so
`krs cases` runs and checks it; nothing else reads the vectors.

    singlestep-cases.py SUITE_DIR OUT_DIR [FILE ...]

writes OUT_DIR/<file>.json for each <file>.json of the suite (all of them by default): a document
of named cases, named as the suite names its vectors.

The mapping, and what the suite does not state:
- The suite treats all 64 KiB as RAM, so the ROM region is empty and `memory` holds the listed
  bytes. `IE` ($FFFF) and `IF` ($FF0F), which every boundary reads, are added as zero where a
  vector does not list them: no interrupt is requested, and the suite models none.
- `F` is a byte in the suite and four flags in the definition (bits 7 to 4; the suite never sets
  bits 3 to 0).
- The suite does not state `EI`'s pending state, `HALT` mode or the `HALT` bug at the start; they
  start clear. `W` and `Z` are unobservable, so their initial value is a choice (`--wz`).
- An idle cycle is compared by kind: the suite records the address and data lines left on the
  bus, which the definition does not model.
- The expected memory is the initial memory with the final bytes written over it, so a byte the
  suite lists only initially is expected unchanged.
- A step runs from the boundary to the next boundary rule: stop before a boundary rule, count 2
  (the first stop is the initial state). The next boundary fetches, except after `HALT`, which
  enters HALT mode. The suite models `HALT` ($76) and `STOP` ($10) as one-cycle no-ops with two
  idle cycles; the definition does not, so those 2000 vectors are expected to fail.
"""

import json
import sys
from pathlib import Path

REGISTERS = ["a", "b", "c", "d", "e", "h", "l"]
IE, IF = 0xFFFF, 0xFF0F
BOUNDARY = [f"crate::interrupt::boundary_{rule}" for rule in ("dispatch", "fetch", "wake", "halted")]
COMPARED = ["regs", "f", "pc", "sp", "ime", "ei_pending", "memory", "bus"]


def flags(byte):
    return {"Flags": {"z": bool(byte & 0x80), "n": bool(byte & 0x40), "h": bool(byte & 0x20), "c": bool(byte & 0x10)}}


def registers(state):
    return {name.upper(): state[name] for name in REGISTERS}


def memory(entries):
    return {"$map": [[address, value] for address, value in sorted(entries.items())]}


def event(cycle):
    address, value, kind = cycle
    if kind == "r-m":
        return {"Read": {"addr": address, "value": value}}
    if kind == "-wm":
        return {"Write": {"addr": address, "value": value}}
    if kind == "---":
        return "Idle"
    raise ValueError(f"unknown cycle kind {kind!r}")


def case(vector, wz):
    initial, final = vector["initial"], vector["final"]
    ram = {address: value for address, value in initial["ram"]}
    for address in (IE, IF):
        ram.setdefault(address, 0)
    expected = dict(ram)
    expected.update({address: value for address, value in final["ram"]})
    halt_entered = ram[initial["pc"]] == 0x76
    return {
        "name": vector["name"],
        "inputs": {
            "regs": registers(initial),
            "f": flags(initial["f"]),
            "pc": initial["pc"],
            "sp": initial["sp"],
            "ime": initial["ime"] != 0,
            "ei_pending": False,
            "halted": False,
            "halt_bug": False,
            "w": wz[0],
            "z": wz[1],
            "rom": {"$hex": ""},
            "memory": memory(ram),
        },
        "expect": {
            "halt": "stopped",
            "rule": BOUNDARY[3] if halt_entered else BOUNDARY[1],
            "compare": COMPARED,
            "cells": {
                "regs": registers(final),
                "f": flags(final["f"]),
                "pc": final["pc"],
                "sp": final["sp"],
                "ime": final["ime"] != 0,
                "ei_pending": final.get("ei", 0) != 0,
                "memory": memory(expected),
                "bus": [event(cycle) for cycle in vector["cycles"]],
            },
        },
    }


def main(argv):
    args = argv[1:]
    wz = (0, 0)
    if args[:1] == ["--wz"]:
        wz = tuple(int(part, 0) for part in args[1].split(","))
        args = args[2:]
    if len(args) < 2:
        sys.exit(__doc__)
    suite, out, files = Path(args[0]), Path(args[1]), args[2:]
    names = files or sorted(path.stem for path in suite.glob("*.json"))
    out.mkdir(parents=True, exist_ok=True)
    run = {"stop_before": BOUNDARY, "stop_count": 2, "depth": 64}
    for name in names:
        vectors = json.loads((suite / f"{name}.json").read_text())
        document = {"run": run, "case": [case(vector, wz) for vector in vectors]}
        (out / f"{name}.json").write_text(json.dumps(document, separators=(",", ":")) + "\n")
    print(f"wrote {len(names)} case documents to {out}", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv)

; DAA after ADD, SUB, half-carry, $99 and $9A; CPL, SCF, CCF
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld a, $45
    add a, $38
    daa
    sub a, $19
    daa
    ld a, $99
    add a, 0
    daa
    add a, 1
    daa
    ld a, 9
    add a, 8
    daa
    cpl
    scf
    ccf
    ccf
    DUMP_AND_LOCK

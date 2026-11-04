; Each ALU operation followed by PUSH AF, so its flags reach the bus
; (seed case of the first mutation run, round 2: flags reach the bus through PUSH AF)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld b, $27
    ld a, $F0
    add a, $0F
    push af
    ld a, $3A
    sub a, b
    push af
    scf
    ld a, $50
    sbc a, $10
    push af
    ld a, $F3
    and a, $35
    push af
    ld a, $20
    cp a, $30
    push af
    DUMP_AND_LOCK

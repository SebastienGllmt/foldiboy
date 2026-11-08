; DAA cases, CPL, CCF, each followed by PUSH AF
; (seed case of the first mutation run, round 2: flags reach the bus through PUSH AF)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld a, $10
    add a, $09
    daa
    push af
    ld a, $99
    add a, 0
    daa
    push af
    ld a, $90
    add a, $90
    daa
    push af
    ld a, $09
    add a, $08
    daa
    push af
    ; DAA after an addition above $99 with no carry, and with a low digit above 9 and no
    ; half-carry; then the same two after a subtraction
    ld a, $50
    add a, $50
    daa
    push af
    ld a, $05
    add a, $05
    daa
    push af
    ld a, $A0
    sub a, $01
    daa
    push af
    ld a, $1F
    sub a, $05
    daa
    push af
    ld a, $5A
    cpl
    push af
    scf
    ccf
    push af
    DUMP_AND_LOCK

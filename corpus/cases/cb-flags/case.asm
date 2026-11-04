; RR with carry set, SLA, SRL to zero, BIT, each followed by PUSH AF
; (seed case of the first mutation run, round 2: flags reach the bus through PUSH AF)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    scf
    ld a, $02
    rr a
    push af
    ld d, $90
    sla d
    push af
    ld e, $01
    srl e
    push af
    ld a, $80
    bit 7, a
    push af
    DUMP_AND_LOCK

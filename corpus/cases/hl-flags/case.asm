; ADD HL,rr with a carry out of bit 7 only and out of bit 11; PUSH HL and AF
; (seed case of the first mutation run, round 2: flags reach the bus through PUSH AF)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld hl, $1080
    ld de, $0080
    add hl, de
    push hl
    push af
    ld hl, $0FFF
    ld bc, $0001
    add hl, bc
    push af
    DUMP_AND_LOCK

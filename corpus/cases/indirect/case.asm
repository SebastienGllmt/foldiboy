; LD A,(BC) / LD (DE),A / LD (C),A / LD A,(C) / LDH (n),A / LDH A,(n) / LD (HL),n
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: the bytes LD A,(BC) and LDH A,(n) read.
    ld a, $A5
    ld [$C030], a
    ld a, $3C
    ldh [$FF87], a

    ld bc, $C030
    ld de, $C031
    ld a, [bc]
    ld [de], a
    ld c, $85
    ldh [c], a
    ld a, 0
    ldh a, [c]
    ldh [$FF86], a
    ldh a, [$FF87]
    ld hl, $C032
    ld [hl], $4E
    DUMP_AND_LOCK

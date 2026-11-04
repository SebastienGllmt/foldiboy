; INC/DEC r and (HL): zero, half-carry, carry preserved
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: the byte INC (HL) and DEC (HL) change.
    ld a, $0F
    ld [$C040], a

    ld b, $FF
    inc b
    ld c, 0
    dec c
    ld d, $10
    dec d
    ld e, 1
    dec e
    inc e
    ld hl, $C040
    inc [hl]
    dec [hl]
    ld a, $7F
    inc a
    dec a
    scf
    inc b
    DUMP_AND_LOCK

; LD r,n / LD r,r / LD (HL),r / LD r,(HL) / LD (HL+),A / LD A,(HL-) / LD (nn),A / LD A,(nn)
; (seed case "loads" of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: the byte LD A,(HL-) reads.
    ld a, $5C
    ld [$C011], a

    ld a, $12
    ld b, a
    ld c, $34
    ld hl, $C010
    ld [hl], c
    ld d, [hl]
    ld [hl+], a
    ld e, $9A
    ld a, [hl-]
    ld [$C020], a
    ld a, [$C010]
    DUMP_AND_LOCK

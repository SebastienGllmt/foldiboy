; CB page: RLC, SRL, SWAP, BIT r and (HL), SET, RES, RR (HL), SLA, SRA, RL; RRCA, RLA, RRA
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: the byte BIT, RR and RES on (HL) use.
    ld a, $5A
    ld [$C060], a

    ld a, $85
    rlc a
    rrca
    rla
    ld b, $81
    srl b
    swap a
    bit 7, a
    ld hl, $C060
    bit 0, [hl]
    set 3, c
    res 7, a
    rr [hl]
    ld d, $90
    sla d
    ld e, $F1
    sra e
    rl c
    rra
    res 0, [hl]
    DUMP_AND_LOCK

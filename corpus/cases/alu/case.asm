; 8-bit ALU with flags: ADD/ADC/SUB/SBC/AND/XOR/OR/CP, register and immediate
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld b, $27
    ld a, $F0
    add a, $0F
    ld a, $3A
    add a, $C6
    adc a, $0F
    sub a, b
    sbc a, $08
    and a, $F0
    xor a, $5A
    or a, b
    cp a, $BF
    cp a, b
    add a, a
    adc a, b
    DUMP_AND_LOCK

; Joypad interrupt (IF bit 4) dispatched to $60
; (seed case of the first mutation run, round 2: flags reach the bus through PUSH AF)
INCLUDE "case.inc"

SECTION "Joypad", ROM0[$0060]
    ld b, $66
    reti

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request the joypad interrupt.
    ld a, $10
    ldh [$FFFF], a
    ldh [$FF0F], a

    ei
    nop
    di
    DUMP_AND_LOCK

; EI delay, dispatch of VBlank then timer, RETI, DI
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "VBlank", ROM0[$0040]
    ld a, $77
    reti

SECTION "Timer", ROM0[$0050]
    inc c
    reti

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request VBlank and timer (IME stays clear until EI).
    ld a, $05
    ldh [$FFFF], a
    ldh [$FF0F], a

    ld a, $11
    ei
    nop
    inc b
    di
    DUMP_AND_LOCK

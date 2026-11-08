; STAT and serial requested together: priority serves STAT, the lower bit, first; its RETI sets
; IME at once, so serial is served at the next boundary. Each handler shifts E into D and writes
; its own number to E, so D and E end as the order of service ($02 then $08).
INCLUDE "case.inc"

SECTION "STAT", ROM0[$0048]
    ld d, e
    ld e, $02
    reti

SECTION "Serial", ROM0[$0058]
    ld d, e
    ld e, $08
    reti

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request STAT and serial (IME stays clear until EI).
    ld de, $0000
    ld a, $0A
    ldh [$FFFF], a
    ldh [$FF0F], a
    ei
    nop
    DUMP_AND_LOCK

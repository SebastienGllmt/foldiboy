; PUSH/POP of BC, DE, HL, AF (F's low nibble masked)
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld bc, $1234
    push bc
    pop de
    ld hl, $ABCD
    push hl
    pop af
    push af
    pop bc
    DUMP_AND_LOCK

; Every RST: each of the 8 vectors holds a handler that writes its own mark, $A0 plus its
; index, to the next byte at HL and returns. A wrong vector writes another mark, or reaches
; an unused byte ($DD) and locks; each RST pushes a different return address.
INCLUDE "case.inc"

; The handler at vector \1 * 8.
MACRO HANDLER
    SECTION "Rst\1", ROM0[(\1) * 8]
        ld a, $A0 + (\1)
        ld [hl+], a
        ret
ENDM

FOR N, 8
    HANDLER {d:N}
ENDR

SECTION "Case", ROM0[$0150]
Start:
    ld hl, $C100
    rst $00
    rst $08
    rst $10
    rst $18
    rst $20
    rst $28
    rst $30
    rst $38
    DUMP_AND_LOCK

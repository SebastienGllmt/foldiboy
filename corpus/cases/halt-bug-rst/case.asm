; The HALT bug on RST: HALT with IME clear and a request pending does not advance PC past the
; RST, so the RST pushes its own address, the handler's RET returns to it, and it runs again,
; this time pushing the next address (Pan Docs, "halt bug"). The handler counts its calls in C: 2.
; (Closes a situation no case reached: `krs kink query`, PATTERNS.toml halt-bug-then-rst.)
INCLUDE "case.inc"

SECTION "Rst08", ROM0[$0008]
    inc c
    ret

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request the timer interrupt; IME is clear.
    ld a, $04
    ldh [$FFFF], a
    ldh [$FF0F], a
    ld c, $00

    halt
    rst $08
    DUMP_AND_LOCK

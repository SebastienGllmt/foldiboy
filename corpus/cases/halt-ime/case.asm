; EI then HALT with a request pending: IME is set when HALT runs, so the CPU does not halt but
; dispatches at once, and the address it pushes is the HALT's own, which the handler pops into HL
; (Pan Docs, "halt"). (sm83-refactor R8: the case the first kink run showed missing.)
INCLUDE "case.inc"

SECTION "Timer", ROM0[$0050]
    pop hl            ; the return address: the HALT
    DUMP_AND_LOCK

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request the timer interrupt; IME is clear.
    ld a, $04
    ldh [$FFFF], a
    ldh [$FF0F], a
    ei
    halt
    ld b, $FF         ; not reached
    LOCK

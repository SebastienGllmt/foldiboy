; EI immediately followed by DI with a request pending: EI takes effect after the next
; instruction, so no interrupt is served between them (Pan Docs, "Interrupts"). The DI runs with
; IME already set and the timer requested, IME ends clear, and the request stays in IF.
; (Closes a situation no case reached: `krs kink query`, PATTERNS.toml ei-then-di.)
INCLUDE "case.inc"

SECTION "Timer", ROM0[$0050]
    ld b, $FF         ; not reached
    LOCK

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request the timer interrupt; IME is clear.
    ld a, $04
    ldh [$FFFF], a
    ldh [$FF0F], a

    ei
    di
    nop               ; a boundary with the request pending and IME clear
    ldh a, [$FF0F]    ; the request is still in IF
    DUMP_AND_LOCK

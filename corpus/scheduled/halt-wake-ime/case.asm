; HALT with IME set, woken by a timer request while halted (case.toml): the CPU leaves HALT mode
; and dispatches the timer interrupt (Pan Docs, "halt"), which acknowledges it in IF.
INCLUDE "case.inc"

SECTION "Timer", ROM0[$0050]
    ld b, $50
    ldh a, [$FF0F]    ; acknowledged: nothing left
    LOCK

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: the timer enabled, nothing requested (the boot ROM leaves VBlank requested).
    ld a, $04
    ldh [$FFFF], a
    xor a
    ldh [$FF0F], a
    ld b, $12
    ei
    nop
    halt
    ld b, $FF         ; not reached: the handler locks
    LOCK

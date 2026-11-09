; HALT with IME clear, woken by a timer request while halted (case.toml): the request ends HALT
; mode whether or not IME is set (Pan Docs, "halt"), and with IME clear the CPU resumes after
; HALT without dispatching, so the request stays in IF.
INCLUDE "case.inc"

SECTION "Timer", ROM0[$0050]
    ld b, $FF         ; not reached: IME is clear
    reti

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: the timer enabled, nothing requested (the boot ROM leaves VBlank requested).
    ld a, $04
    ldh [$FFFF], a
    xor a
    ldh [$FF0F], a
    ld b, $12
    halt
    ld b, $34         ; after the wake
    ldh a, [$FF0F]    ; the timer request, still pending
    LOCK

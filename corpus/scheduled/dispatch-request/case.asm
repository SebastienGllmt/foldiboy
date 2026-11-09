; A request that arrives while an interrupt is being dispatched (case.toml): the timer is
; requested at an instruction boundary with IME set, so the CPU starts its dispatch, and VBlank
; is requested during the dispatch's internal cycles. IF is sampled after both pushes of PC, so
; the vector is VBlank's, the higher priority, and only VBlank is acknowledged.
INCLUDE "case.inc"

SECTION "VBlank", ROM0[$0040]
    ld b, $40
    ldh a, [$FF0F]    ; the timer, still requested
    LOCK

SECTION "Timer", ROM0[$0050]
    ld b, $50         ; not reached: VBlank is served
    LOCK

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: VBlank and the timer enabled, nothing requested (the boot ROM leaves VBlank
    ; requested).
    ld a, $05
    ldh [$FFFF], a
    xor a
    ldh [$FF0F], a
    ld b, $12
    ei
    nop
    nop
    nop
    ld b, $FF         ; not reached: the dispatch comes first
    LOCK

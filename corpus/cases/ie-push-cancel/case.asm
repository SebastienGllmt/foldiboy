; An interrupt cancelled by its own dispatch: with SP at $0000 the first push writes PC's high byte
; to IE ($FFFF), the dispatch reads IE again after that write, no enabled interrupt is requested
; any more, and it jumps to $0000 with IF unchanged (mooneye ie_push; Pan Docs, "Interrupts").
; (sm83-refactor R8: the cases the first kink results showed missing.)
INCLUDE "case.inc"

SECTION "Reset", ROM0[$0000]
    jp Cancelled

SECTION "Timer", ROM0[$0050]
    ld b, $50         ; reached only if the dispatch was not cancelled
    jp Cancelled

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request the timer interrupt; IME is clear.
    ld a, $04
    ldh [$FFFF], a
    ldh [$FF0F], a
    ld sp, $0000      ; PC is $01xx: the high push writes $01 to IE (VBlank only)
    ei
    nop               ; dispatched after this instruction
    ld b, $FF         ; not reached

Cancelled:
    ld sp, $D000
    ldh a, [$FF0F]    ; IF, still $04 if the dispatch acknowledged nothing
    ld c, a
    ldh a, [$FFFF]    ; IE, now $01
    ld d, a
    DUMP_AND_LOCK

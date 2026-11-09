; Requests seen by a read of IF per machine cycle (case.toml): `LDH A,[IF]` reads IF in its third
; machine cycle. A request from that cycle on is seen by the read; one from the next cycle is not,
; and the second read sees both. IME is clear, so nothing is dispatched.
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: VBlank and STAT enabled, nothing requested (the boot ROM leaves VBlank requested).
    ld a, $03
    ldh [$FFFF], a
    xor a
    ldh [$FF0F], a
    ldh a, [$FF0F]    ; cycles 14-16: reads IF at 16
    ld b, a           ; $01: VBlank only
    ldh a, [$FF0F]    ; $03: VBlank and STAT
    LOCK

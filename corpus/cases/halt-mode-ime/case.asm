; HALT with IME set and no interrupt requested: the CPU enters HALT mode, as with IME clear
; (Pan Docs, "halt"). It never wakes: the case ends at its second halted boundary
; (FLOW.toml [stop]).
INCLUDE "case.inc"

SECTION "VBlank", ROM0[$0040]
    ld b, $FF         ; not reached
    reti

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: VBlank enabled, nothing requested (the boot ROM leaves VBlank requested).
    ld a, $01
    ldh [$FFFF], a
    xor a
    ldh [$FF0F], a
    ld b, $12
    ei
    nop
    halt
    ld b, $FF         ; not reached
    LOCK

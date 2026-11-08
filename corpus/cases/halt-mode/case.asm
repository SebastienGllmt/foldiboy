; HALT with IME clear and no interrupt requested: the CPU enters HALT mode and idles at every
; boundary (Pan Docs, "halt"). Nothing at the CPU stage requests an interrupt, so it never
; wakes: the case ends at its second halted boundary (FLOW.toml [stop]).
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: VBlank enabled, nothing requested (the boot ROM leaves VBlank requested).
    ld a, $01
    ldh [$FFFF], a
    xor a
    ldh [$FF0F], a
    ld b, $12
    halt
    ld b, $FF         ; not reached
    LOCK

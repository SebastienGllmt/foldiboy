; CALL / CALL cc / RET / RET cc / RST
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Rst38", ROM0[$0038]
    inc b
    ret

SECTION "Case", ROM0[$0150]
Start:
    call Sub1
    xor a
    call nz, Sub1
    call z, Sub2
    rst $38
    jp After
Sub1:
    inc a
    ret
Sub2:
    ld d, $44
    scf
    ret nc
    ret c
After:
    DUMP_AND_LOCK

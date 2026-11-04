; JR cc taken/not taken, JR backwards loop, JP cc, JP HL, JR
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    xor a
    jr nz, .t0
    jr z, .t0
    inc a
    inc a
.t0
    ld b, 3
.loop
    dec b
    jr nz, .loop
    jp nc, .t1
    inc a
.t1
    scf
    jp nc, .t1
    jp c, .t2
    inc a
.t2
    ld hl, .t3
    jp hl
    inc a
.t3
    jr .t4
    inc a
.t4
    DUMP_AND_LOCK

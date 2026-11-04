; LD rr,nn / INC rr / DEC rr / ADD HL,rr / ADD SP,e / LD HL,SP+e / LD SP,HL / LD (nn),SP
; (seed case of the first mutation run, draft-personal/observer-design/measurements/kill-power.md)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld bc, $12FF
    inc bc
    ld de, $0FFF
    ld hl, $F001
    add hl, de
    dec de
    add hl, bc
    add sp, -4
    ld hl, sp+3
    ld sp, hl
    inc sp
    dec sp
    ld [$C050], sp
    DUMP_AND_LOCK

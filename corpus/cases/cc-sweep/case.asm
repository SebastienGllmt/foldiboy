; Every condition code of JR cc, JP cc, CALL cc and RET cc (NZ, Z, NC, C), each in all four
; states of the Z and C flags, so each is taken twice and not taken twice.
;
; Each test pops its flags and a mark (twice its index) into AF, runs the conditional, and
; writes A to the next byte at HL: the taken JR, JP or CALL adds one to the mark, and the RET
; not taken adds one. Two conditions differ in at least one of the four states, so a wrong
; condition writes a wrong mark; the marks of different tests never coincide.
INCLUDE "case.inc"

DEF TEST = 0

; AF = the mark of the next test, with Z = \1 and C = \2 (N and H clear).
MACRO FLAGS
    ld bc, ((TEST * 2) << 8) | ((\1) << 7) | ((\2) << 4)
    push bc
    pop af
    DEF TEST += 1
ENDM

MACRO JR_TEST
    FLAGS \2, \3
    jr \1, .taken\@
    jr .log\@
.taken\@
    inc a
.log\@
    ld [hl+], a
ENDM

MACRO JP_TEST
    FLAGS \2, \3
    jp \1, .taken\@
    jr .log\@
.taken\@
    inc a
.log\@
    ld [hl+], a
ENDM

MACRO CALL_TEST
    FLAGS \2, \3
    call \1, IncA
    ld [hl+], a
ENDM

MACRO RET_TEST
    FLAGS \2, \3
    call Ret_\1
    ld [hl+], a
ENDM

; Test kind \1 with condition \2 in the four flag states.
MACRO STATES
    \1 \2, 0, 0
    \1 \2, 0, 1
    \1 \2, 1, 0
    \1 \2, 1, 1
ENDM

SECTION "Case", ROM0[$0150]
Start:
    ld hl, $C100
    STATES JR_TEST, nz
    STATES JR_TEST, z
    STATES JR_TEST, nc
    STATES JR_TEST, c
    STATES JP_TEST, nz
    STATES JP_TEST, z
    STATES JP_TEST, nc
    STATES JP_TEST, c
    STATES CALL_TEST, nz
    STATES CALL_TEST, z
    STATES CALL_TEST, nc
    STATES CALL_TEST, c
    STATES RET_TEST, nz
    STATES RET_TEST, z
    STATES RET_TEST, nc
    STATES RET_TEST, c
    DUMP_AND_LOCK

IncA:
    inc a
    ret

; RET cc, then the not-taken path adds one before returning.
Ret_nz:
    ret nz
    inc a
    ret
Ret_z:
    ret z
    inc a
    ret
Ret_nc:
    ret nc
    inc a
    ret
Ret_c:
    ret c
    inc a
    ret

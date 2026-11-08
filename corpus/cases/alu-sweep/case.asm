; Every 8-bit ALU encoding: the 8 operations (ADD ADC SUB SBC AND XOR OR CP) on the 8 operands
; (B, C, D, E, H, L, (HL), A), and on an immediate. Each test loads A, runs the operation and
; pushes AF.
;
; The operands are B = $12, C = $34, D = $56, E = $78, H = $CB, L = $9D, (HL) = $AF: their
; high nibbles differ, their low nibbles differ, and none is zero. Each operation's A
; ($F1 ADD, $E2 ADC, $63 SUB, $74 SBC, $F0 AND, $E7 XOR, $0F OR) shares a bit with every
; operand, and the four arithmetic operations run with the carry set, so on every operand a
; wrong operand or any other operation gives a different A or flags. CP leaves A as it was:
; each CP compares A with an equal operand (Z set), so another operand clears Z.
; A register taken for (HL), or (HL) for a register, changes the bus.
INCLUDE "case.inc"

; \1 A, \2 with A = \3; with the carry set first when \4 is 1.
MACRO TEST
    IF \4
        scf
    ENDC
    ld a, \3
    \1 a, \2
    push af
ENDM

; Operation \1 on every operand and on the immediate $69, with A = \2.
MACRO OP_GROUP
    TEST \1, b, \2, \3
    TEST \1, c, \2, \3
    TEST \1, d, \2, \3
    TEST \1, e, \2, \3
    TEST \1, h, \2, \3
    TEST \1, l, \2, \3
    TEST \1, [hl], \2, \3
    TEST \1, a, \2, \3
    TEST \1, $69, \2, \3
ENDM

SECTION "Case", ROM0[$0150]
Start:
    ; The stack is in WRAM: the sweep pushes more than HRAM holds.
    ld sp, $E000
    ld bc, $1234
    ld de, $5678
    ld hl, $CB9D
    ld [hl], $AF

    OP_GROUP add, $F1, 1
    OP_GROUP adc, $E2, 1
    OP_GROUP sub, $63, 1
    OP_GROUP sbc, $74, 1
    OP_GROUP and, $F0, 0
    OP_GROUP xor, $E7, 0
    OP_GROUP or, $0F, 0
    ; CP with A equal to the operand
    TEST cp, b, $12, 0
    TEST cp, c, $34, 0
    TEST cp, d, $56, 0
    TEST cp, e, $78, 0
    TEST cp, h, $CB, 0
    TEST cp, l, $9D, 0
    TEST cp, [hl], $AF, 0
    TEST cp, a, $E9, 0
    TEST cp, $69, $69, 0
    DUMP_AND_LOCK

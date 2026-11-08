; Every CB opcode: the 8 rotations and shifts, BIT, RES and SET, each on the 8 operands
; (B, C, D, E, H, L, (HL), A), so every value of each decode_cb_* field is executed.
;
; Why each encoding is told apart:
; - A rotation group gives its 8 operands different values (bit 7 and bit 0 both vary, none a
;   fixed point of the rotation) and pushes AF after each operation: a wrong operand leaves the
;   intended one unchanged, and every other rotation gives a different result or carry on some
;   operand of the group.
; - BIT y tests a walking one: before each test every register is zero except the operand,
;   which holds only bit y, and XOR A has set Z. A wrong bit index or a wrong register leaves
;   Z set; PUSH AF records it.
; - A RES y (SET y) group gives every operand a different value with bit y set (clear) and
;   pushes every register at its end: a wrong bit index or a wrong register gives another value.
; - A register taken for (HL), or (HL) for a register, changes the bus. Each group's (HL)
;   operand is at OPERANDS plus the opcode of its (HL) form, so every result stays in memory.
;   H and L are operated on after the (HL) form, which needs them as the pointer.
INCLUDE "case.inc"

DEF OPERANDS EQU $C200

; Rotation group: \1 is the mnemonic, \2 its row y. Starts with the carry set.
MACRO ROT_GROUP
    ld bc, $42C4
    ld de, $23AC
    ld hl, OPERANDS + (((\2) << 3) | 6)
    ld [hl], $81
    ld a, $59
    scf
    \1 [hl]
    push af
    \1 b
    push af
    \1 c
    push af
    \1 d
    push af
    \1 e
    push af
    \1 a
    push af
    \1 h
    push af
    \1 l
    push af
    push bc
    push de
    push hl
ENDM

; BIT \1 on register \2, which holds the only set bit; A is zero.
MACRO BIT_REG
    xor a
    ld \2, 1 << (\1)
    bit \1, \2
    push af
    ld \2, 0
ENDM

; BIT group: row \1.
MACRO BIT_GROUP
    ld bc, 0
    ld de, 0
    ld hl, OPERANDS + ($46 | ((\1) << 3))
    xor a
    ld [hl], 1 << (\1)
    bit \1, [hl]
    push af
    ld hl, 0
    BIT_REG \1, b
    BIT_REG \1, c
    BIT_REG \1, d
    BIT_REG \1, e
    BIT_REG \1, h
    BIT_REG \1, l
    xor a
    ld a, 1 << (\1)
    bit \1, a
    push af
ENDM

; The body of a RES or SET group: \1 is the mnemonic, \2 the row y, \3 the opcode of its (HL)
; form. The caller has loaded B, C, D, E and A, and defined VH, VL and VM, the values of H, L
; and (HL).
MACRO BITOP_BODY
    ld hl, OPERANDS + (\3)
    ld [hl], VM
    \1 \2, [hl]
    \1 \2, b
    \1 \2, c
    \1 \2, d
    \1 \2, e
    \1 \2, a
    ld hl, (VH << 8) | VL
    \1 \2, h
    \1 \2, l
    push af
    push bc
    push de
    push hl
    PURGE VH, VL, VM
ENDM

; RES group: row \1. Every operand has bit y set.
MACRO RES_GROUP
    DEF M = 1 << (\1)
    ld bc, (($12 | M) << 8) | ($34 | M)
    ld de, (($56 | M) << 8) | ($78 | M)
    ld a, $9A | M
    DEF VH = $BC | M
    DEF VL = $DE | M
    DEF VM = $F0 | M
    BITOP_BODY res, \1, $86 | ((\1) << 3)
    PURGE M
ENDM

; SET group: row \1. Every operand has bit y clear.
MACRO SET_GROUP
    DEF M = 1 << (\1)
    ld bc, (($12 & ~M) << 8) | ($34 & ~M)
    ld de, (($56 & ~M) << 8) | ($78 & ~M)
    ld a, $9A & ~M
    DEF VH = $BC & ~M
    DEF VL = $DE & ~M
    DEF VM = $F0 & ~M
    BITOP_BODY set, \1, $C6 | ((\1) << 3)
    PURGE M
ENDM

SECTION "Case", ROM0[$0150]
Start:
    ; The stack is in WRAM: the sweep pushes more than HRAM holds.
    ld sp, $E000

    ROT_GROUP rlc, 0
    ROT_GROUP rrc, 1
    ROT_GROUP rl, 2
    ROT_GROUP rr, 3
    ROT_GROUP sla, 4
    ROT_GROUP sra, 5
    ROT_GROUP swap, 6
    ROT_GROUP srl, 7
    FOR Y, 8
        BIT_GROUP Y
    ENDR
    FOR Y, 8
        RES_GROUP Y
    ENDR
    FOR Y, 8
        SET_GROUP Y
    ENDR
    DUMP_AND_LOCK

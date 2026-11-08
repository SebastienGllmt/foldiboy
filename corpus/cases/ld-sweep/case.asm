; Every LD r, r' (the 63 encodings of 01 yyy zzz other than HALT): for each destination, a
; load from every source, from the same distinct values B = $12, C = $34, D = $56, E = $78,
; H = $C3, L = $5A, (HL) = $E1, A = $9A.
;
; After each load the destination's pair is pushed, so a wrong source shows a wrong value.
; The load from the destination itself comes first, while the destination still holds its own
; value, which no other source holds. Every register is pushed at the end of each group, so a
; wrong destination shows too. Loads to (HL) write the bus; a register taken for (HL), or (HL)
; for a register, changes the bus. Loads into H and L come after the load from (HL).
INCLUDE "case.inc"

MACRO LOAD_ALL
    ld bc, $1234
    ld de, $5678
    ld hl, $C35A
    ld a, $9A
ENDM

; LD \1, \3 then PUSH \2, the pair that holds \1.
MACRO LOAD_PUSH
    ld \1, \3
    push \2
ENDM

; Every load into register \1, whose pair is \2: from \1 itself, from (HL), then from the six
; other registers \3 to \8.
MACRO DEST_GROUP
    LOAD_ALL
    LOAD_PUSH \1, \2, \1
    LOAD_PUSH \1, \2, [hl]
    LOAD_PUSH \1, \2, \3
    LOAD_PUSH \1, \2, \4
    LOAD_PUSH \1, \2, \5
    LOAD_PUSH \1, \2, \6
    LOAD_PUSH \1, \2, \7
    LOAD_PUSH \1, \2, \8
    push af
    push bc
    push de
    push hl
ENDM

SECTION "Case", ROM0[$0150]
Start:
    ; The stack is in WRAM: the sweep pushes more than HRAM holds.
    ld sp, $E000
    ld hl, $C35A
    ld [hl], $E1

    DEST_GROUP b, bc, c, d, e, h, l, a
    DEST_GROUP c, bc, b, d, e, h, l, a
    DEST_GROUP d, de, b, c, e, h, l, a
    DEST_GROUP e, de, b, c, d, h, l, a
    DEST_GROUP h, hl, b, c, d, e, l, a
    DEST_GROUP l, hl, b, c, d, e, h, a
    DEST_GROUP a, af, b, c, d, e, h, l

    ; Every store to (HL), last: it overwrites the byte the loads from (HL) read.
    LOAD_ALL
    ld [hl], b
    ld [hl], c
    ld [hl], d
    ld [hl], e
    ld [hl], h
    ld [hl], l
    ld [hl], a
    DUMP_AND_LOCK

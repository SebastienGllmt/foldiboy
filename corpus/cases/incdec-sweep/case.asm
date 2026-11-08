; Every INC r and DEC r (B, C, D, E, H, L, (HL), A) and every INC rr and DEC rr (BC, DE, HL,
; SP).
;
; Each 8-bit group gives its 8 operands different values, pushes AF after each operation (Z, N
; and H) and pushes every register at its end: a wrong operand leaves the intended one
; unchanged and changes another twice. The inputs include $FF (INC to zero), $0F (INC with
; half-carry), $01 (DEC to zero), $10 (DEC with half-borrow) and $00 (DEC to $FF). H and L
; come after (HL), which needs them as the pointer; a register taken for (HL), or (HL) for a
; register, changes the bus.
;
; The 16-bit operations start from different values that carry or borrow across the byte, and
; every pair is pushed (SP stored with LD (nn), SP) after each group.
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; The stack is in WRAM, as the 16-bit group moves SP.
    ld sp, $E000

    ld bc, $FF0F
    ld de, $5678
    ld hl, $C35A
    ld [hl], $E1
    ld a, $9A
    inc [hl]
    push af
    inc b
    push af
    inc c
    push af
    inc d
    push af
    inc e
    push af
    inc a
    push af
    inc h
    push af
    inc l
    push af
    push bc
    push de
    push hl

    ld bc, $0110
    ld de, $00A7
    ld hl, $C45B
    ld [hl], $3E
    ld a, $D2
    dec [hl]
    push af
    dec b
    push af
    dec c
    push af
    dec d
    push af
    dec e
    push af
    dec a
    push af
    dec h
    push af
    dec l
    push af
    push bc
    push de
    push hl

    ld bc, $12FF
    ld de, $3456
    ld hl, $78FF
    inc bc
    inc de
    inc hl
    push bc
    push de
    push hl
    inc sp
    ld [$C010], sp

    ld bc, $9A00
    ld de, $BC01
    ld hl, $DE00
    dec bc
    dec de
    dec hl
    dec sp
    ld [$C012], sp
    push bc
    push de
    push hl
    DUMP_AND_LOCK

; Every LD rr, nn and ADD HL, rr (BC, DE, HL, SP), and every LD (rr), A and LD A, (rr) (BC, DE,
; HL+, HL-).
;
; The pairs hold different values, and HL and the flags are pushed after each ADD HL, so a
; wrong pair gives a different sum. The stores write a different byte through each of four
; pointers ($C110, $C120, $C130 and, after HL+, $C131), and the loads read them back through
; the same encodings, each followed by PUSH AF: a wrong pointer reads another byte, or one never
; written, and HL+ taken for HL- (or the reverse) moves HL the other way.
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld sp, $E000
    ld bc, $1234
    ld de, $5678
    ld hl, $8A23
    add hl, bc
    push hl
    push af
    add hl, de
    push hl
    push af
    add hl, hl
    push hl
    push af
    add hl, sp
    push hl
    push af

    ld bc, $C110
    ld de, $C120
    ld hl, $C130
    ld a, $A1
    ld [bc], a
    ld a, $A2
    ld [de], a
    ld a, $A3
    ld [hl+], a
    ld a, $A4
    ld [hl-], a
    push hl
    ld a, [bc]
    push af
    ld a, [de]
    push af
    ld a, [hl+]
    push af
    ld a, [hl-]
    push af
    push hl
    DUMP_AND_LOCK

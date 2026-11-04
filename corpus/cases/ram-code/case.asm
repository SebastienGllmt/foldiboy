; Code in RAM: a routine copied to WRAM and called there, so every fetch, a CB-prefixed fetch,
; the HALT bug's repeated fetch and an interrupt dispatch's discarded opcode read are RAM reads,
; not ROM reads. Also RLCA, RRC, and H and L as operands. (sm83-refactor R8: the cases the first
; kink results showed missing.)
INCLUDE "case.inc"

SECTION "Timer", ROM0[$0050]
    inc c
    reti

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: copy the routine to $C100.
    ld hl, $C100
    ld de, RamCode
    ld c, RamCodeEnd - RamCode
.copy
    ld a, [de]
    ld [hl+], a
    inc de
    dec c
    jr nz, .copy

    call RamRoutine
    DUMP_AND_LOCK

RamCode:
LOAD "Routine", WRAM0[$C100]
RamRoutine:
    ld h, $12
    ld l, $B4
    ld a, h
    add a, l          ; $C6: A + L
    rlca              ; $8D, carry set
    ld b, a
    rrc b             ; a CB instruction fetched from RAM
    ; Enable and request the timer interrupt with IME clear: HALT does not halt, and the
    ; next byte is fetched twice.
    ld a, $04
    ldh [$FFFF], a
    ldh [$FF0F], a
    halt
    inc a
    ; IME is set after the NOP, and the timer interrupt is dispatched from here, in RAM.
    ei
    nop
    ret
ENDL
RamCodeEnd:

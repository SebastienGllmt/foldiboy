; The HALT bug on a two-byte instruction: HALT with IME clear and a request pending does not
; advance PC past the next opcode, so LD A, n reads its own opcode ($3E) as its operand, and its
; operand byte ($14) then executes as INC D (Pan Docs, "halt bug").
; (Closes a situation no case reached: `krs kink query`, PATTERNS.toml halt-bug-on-two-byte-opcode.)
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ; Prologue: enable and request the timer interrupt; IME is clear.
    ld a, $04
    ldh [$FFFF], a
    ldh [$FF0F], a
    ld d, $00

    halt
    ld a, $14         ; runs as LD A, $3E, then INC D
    DUMP_AND_LOCK

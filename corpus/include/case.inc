; Shared by every corpus case (draft-personal/corpus-contract D-CASE-ROM, D-CASE-END).
;
; A case is a cartridge: execution starts at $0100 in the state the DMG boot ROM leaves, jumps
; over the header (rgbfix fills it) to Start, which the case defines, and ends by locking the CPU.
; RAM holds no byte until the case writes one: read only what you wrote.

SECTION "Entry", ROM0[$0100]
    nop
    jp Start

SECTION "Header", ROM0[$0104]
    ds $0150 - $0104, 0

; Lock the CPU: $DD is one of the eleven unused opcodes; every system ends the case here.
MACRO LOCK
    db $DD
ENDM

; Write every register to the stack, and SP to $C000-$C001, then lock: makes the end state visible
; to a system that reports only memory or the bus (hardware). Systems that report their end state
; need it only for flags, which no other instruction writes to the bus.
MACRO DUMP_AND_LOCK
    push af
    push bc
    push de
    push hl
    ld [$C000], sp
    LOCK
ENDM

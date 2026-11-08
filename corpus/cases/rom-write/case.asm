; A write into the ROM region: the bus carries it, and the ROM keeps its byte (a cartridge
; without a mapper; D-MEMORY), so the read back gives the ROM's value.
INCLUDE "case.inc"

SECTION "Data", ROM0[$0200]
Data:
    db $5A

SECTION "Case", ROM0[$0150]
Start:
    ld a, $A5
    ld [Data], a      ; ignored by the ROM
    ld a, [Data]      ; $5A
    DUMP_AND_LOCK

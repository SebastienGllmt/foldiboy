; STOP: the CPU stage does not model it (it waits for a joypad press and stops the clocks), so
; the definition decodes it and has no rule to execute it: the run ends stuck on STOP.
INCLUDE "case.inc"

SECTION "Case", ROM0[$0150]
Start:
    ld b, $12
    stop
    ld b, $FF         ; not reached
    LOCK

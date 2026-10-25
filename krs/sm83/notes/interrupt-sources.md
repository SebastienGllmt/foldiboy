# Interrupt, HALT, EI/DI and STOP: sources

Collected by a sub-agent (AI-written, 2026-10-20) from local clones; every claim cites `file:line`. Spot-checked: SameBoy `halt()`, `ei()` and the dispatch path (`Core/sm83_cpu.c:1030-1050`, `1340-1360`, `1620-1720`), and the tests listed as passing in `README.md`.

Prefixes, all under `~/git/repos/github.com/`: PD = `gbdev/pandocs/src`, CTR = `Gekkio/gb-ctr`, MTS = `Gekkio/mooneye-test-suite` (also in `~/git/workspaces/foldiboy/roms/tests/`), SB = `LIJI32/SameBoy/Core`, MGB = `Gekkio/mooneye-gb/core/src`, GAM = `gb-archive/gambatte/libgambatte/src`.
gb-ctr has no section on interrupt dispatch, and its HALT and STOP sections are TODO (CTR/chapter/cpu/instruction-set.typ:4524-4530).

## Dispatch

- 5 M-cycles: PD/Interrupts.md:67-72; MTS/acceptance/intr_timing.s:21-43.
- SameBoy: M1 reads the opcode at PC and discards it (pc++ then pc--), M2 and M3 are internal, M4 writes PC high to SP-1, M5 writes PC low to SP-2 (SB/sm83_cpu.c:1663-1699). The PC undo is also in GAM/interrupter.cpp:41-45 and MGB/cpu.rs:74-82.
- IE is sampled after the high-byte push and IF after the low-byte push (SB:1672, 1680). MGB and GAM differ only when IF changes during M5.
- IF is acknowledged and PC set after the pushes: SB:1689-1694, MGB/cpu.rs:137-146, GAM/interrupter.cpp:62-63. Pan Docs' order (PD/Interrupts.md:58-63) is contradicted by `ie_push`.
- IME is cleared even on cancellation, and PC becomes $0000: MTS/acceptance/interrupts/ie_push.s:37-69; SB:1696-1699.
- The lowest bit is served, at vector $40 + 8·bit, and only bits 0-4 count: PD/Interrupts.md:82-86; SB:1629, 1684-1694.
- IF bits 7-5 read as 1 (MTS if_ie_registers.s:69-72, bits/unused_hwio-GS.s:92-93). IE is a full 8-bit register (unused_hwio-GS.s:108-109).
- IE and IF are not read over the external bus: IE is in the CPU (CTR/chapter/cpu/simple.typ:19). This is inferred from the model; no source says it in those words.

## EI, DI, RETI

- EI takes effect one instruction late: PD/Interrupts.md:15-16; CTR instruction-set.typ:4567, 4584-4589; MTS ei_timing.s:38-47.
- EI followed by DI takes no interrupt (MTS rapid_di_ei.s:43-65). EI followed by EI takes the interrupt after the second EI (MTS ei_sequence.s:37-50). SB:1352-1358 arms EI only when IME=0.
- DI is immediate on DMG (MTS di_timing-GS.s:24-26). On CGB, mooneye says DI is delayed and SameBoy says it is not (SB:1347-1348).
- RETI sets IME immediately: CTR:4459, 4476; MTS reti_intr_timing.s:38-52.

## HALT

- IME=1: wakes and dispatches (PD/halt.md:98-106).
- IME=0 with nothing pending: resumes without dispatch (PD/halt.md:111-113; SB:1642-1651).
- IME=0 with an interrupt pending: the HALT bug, so the next byte is read twice (PD/halt.md:117-130; SB:1044-1046, 1711-1714).
- IME=1 with an interrupt pending, including EI followed by HALT: returns into the HALT. Sources: SB:1040-1042, GAM cpu.cpp:1019-1022, and SameSuite interrupt/ei_delay_halt.asm:34-38. mooneye-gb instead returns to HALT+1 (MGB/execute.rs:550-553).
- On DMG, halted time advances one M-cycle per step with no extra wake-up cycle (MGB/cpu.rs:150-156; SB:1625-1654). GAM adds a wake-up cycle on CGB (memory.cpp:251).

## STOP (low certainty)

- Ended by a P10-P13 input going low (PD/Reducing_Power_Consumption.md:78-81).
- Resets DIV (PD/Timer_and_Divider_Registers.md:16-17).
- 1 or 2 bytes, and HALT mode or STOP mode, depending on whether a button is held and whether an interrupt is pending (PD/imgs/stop_diagram.drawio; SB:385-453).
- mooneye-gb panics on STOP, and gambatte marks it FIXME.

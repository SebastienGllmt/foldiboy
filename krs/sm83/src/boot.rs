//! Boot states: the architectural state a hardware model leaves when its boot
//! ROM hands over to the cartridge at `$0100`, stated once for every run that
//! starts from a cartridge (`Machine::init`).
//!
//! A ROM case's initial state is a pure function of the cartridge and the
//! model: these values, the cartridge's first 32 KiB as the ROM region
//! (`RomRegion`), and RAM that holds no byte until the program writes one.

use k::Term;
use k::builtin::{Bytes, Int, Map, bytes, int};

use crate::alu::{FlagSet, Flags};
use crate::interrupt::{IE, IF};
use crate::machine::{Reg8, Register};

/// The ROM region of a cartridge: its first 32 KiB, or all of a smaller one.
/// Above `$8000` the address space is RAM and I/O.
#[k::function(total, internal)]
pub fn RomRegion(cartridge: Term<Bytes>) -> Term<Bytes> {
    bytes::substr(cartridge, 0, int::min(bytes::length(cartridge), 0x8000))
}

/// DMG (Pan Docs, "Power Up Sequence"): A B C D E H L after the boot ROM.
#[k::function(total, internal)]
pub fn DmgRegisters() -> Term<Map<Reg8, Int>> {
    k::map![
        Register::A -> 0x01,
        Register::B -> 0x00,
        Register::C -> 0x13,
        Register::D -> 0x00,
        Register::E -> 0xD8,
        Register::H -> 0x01,
        Register::L -> 0x4D,
    ]
}

/// DMG: F is `$B0` (Z, H and C set) after the boot ROM.
#[k::function(total, internal)]
pub fn DmgFlags() -> Term<FlagSet> {
    Flags {
        z: true,
        n: false,
        h: true,
        c: true,
    }
}

/// Every model hands over at `$0100`.
#[k::function(total, internal)]
pub fn BootPc() -> Term<Int> {
    0x0100
}

/// Every model hands over with `SP` at `$FFFE`.
#[k::function(total, internal)]
pub fn BootSp() -> Term<Int> {
    0xFFFE
}

/// The interrupt registers after the DMG boot ROM: `IF` is `$E1`, `IE` zero.
#[k::function(total, internal)]
pub fn DmgIo() -> Term<Map<Int, Int>> {
    k::map![IF -> 0xE1, IE -> 0x00]
}

/// `DmgIo` for the CPU stage, which models no display. Two stand-ins, by
/// which test ROMs (mooneye) skip their display code: `LCDC` (`$FF40`) reads
/// zero (LCD off) and `LY` (`$FF44`) reads `$FF`. Neither is a hardware
/// value; the console stage replaces both with the PPU. Any other I/O register
/// the CPU stage does not model holds no byte, so a read of it is stuck.
#[k::function(total, internal)]
pub fn CpuStageIo() -> Term<Map<Int, Int>> {
    k::map![0xFF40 -> 0x00, 0xFF44 -> 0xFF, ..DmgIo()]
}

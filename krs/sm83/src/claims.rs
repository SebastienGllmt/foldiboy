//! Authored native contracts; see `claims/README.md` for behavior sources and scope.

pub mod boundaries;
pub mod control;
pub mod decode;
pub mod fetch;
pub mod instructions;
pub mod memory;

#[k::spec_module(name = "SM83-ADD-HL-DE-SPEC", imports = [crate])]
pub mod add_hl_de {
    use crate::Machine;
    use crate::alu::Flags;
    use crate::decode::{Instruction, Pair16};
    use crate::machine::{Event, Register};
    use crate::well_formed::IsByte;

    k::claims! {
        for Machine;

        /// ADD HL,DE for arbitrary byte registers and incoming flags.
        /// Specify wrapping and carries arithmetically, independently of the ALU helpers.
        /// The half-carry and carry compare sums with 4096 and 65536.
        /// Preserve Z, clear N, and record the one remaining internal cycle.
        /// Start at the decoded instruction, after fetch and arbitration.
        #[k::claim(all_paths)]
        #[k::requires(
            IsByte(r[Register::H])
                && IsByte(r[Register::L])
                && IsByte(r[Register::D])
                && IsByte(r[Register::E])
        )]
        add_hl_de: {
            k: [
                Instruction::AddHl {
                    pair: Pair16::Registers { high: Register::D, low: Register::E },
                },
            ],
            regs: r,
            f: Flags { z: zero, .. },
            bus: log,
        } => {
            k: [],
            regs: k::map![
                Register::H -> (
                    (
                        r[Register::H] * 0x100 + r[Register::L] + r[Register::D] * 0x100 + r[
                            Register::E
                        ]
                    ) % 0x1_0000
                ) / 0x100,
                Register::L -> (
                    r[Register::H] * 0x100 + r[Register::L] + r[Register::D] * 0x100 + r[
                        Register::E
                    ]
                ) % 0x100,
                ..r,
            ],
            f: Flags {
                z: zero,
                n: false,
                h: (r[Register::H] * 0x100 + r[Register::L]) % 0x1000
                    + (r[Register::D] * 0x100 + r[Register::E]) % 0x1000
                    >= 0x1000,
                c: r[Register::H] * 0x100 + r[Register::L] + r[Register::D] * 0x100 + r[Register::E]
                    >= 0x1_0000,
            },
            bus: [..log, Event::Idle],
        };
    }
}

/// Compose every authored contract through the native KRS spec-module imports.
#[k::spec_module(name = "SM83-CONTRACTS-SPEC", imports = [
    crate,
    crate::claims::add_hl_de,
    crate::claims::instructions::spec,
    crate::claims::decode::spec,
    crate::claims::memory::spec,
    crate::claims::control::spec,
    crate::claims::fetch::spec,
    crate::claims::boundaries::spec,
])]
pub mod all {}

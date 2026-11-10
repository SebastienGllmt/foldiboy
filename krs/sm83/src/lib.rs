//! SM83 instruction-set semantics.
//!
//! One instruction is one step.
//! A step is a sequence of micro-operations on the computation cell.
//! Each machine cycle of the instruction is one bus micro-operation, which
//! appends one entry to the bus log; a value read in one cycle and used in
//! a later one passes through the CPU's internal `W` and `Z` registers.

pub use sm83_core::{alu, arith};
pub mod boot;
pub mod claims;
pub mod decode;
pub mod execute;
pub mod interrupt;
pub mod machine;
pub mod micro;
pub mod well_formed;

pub use alu::{
    AddHlFlags, AluFlags, AluResult, BitFlags, ByteFlags, CarryIn, CarrySet, DaaCorrection,
    DaaHigh, DaaLow, FlagByte, FlagSet, Flags, HalfCarrySet, ShiftFlags, SpOffsetFlags,
    SubtractSet, UnaryFlags, UnaryResult, ZeroSet,
};
pub use arith::{Byte, FlagBit, High, Low, Pair, Signed, Word};
pub use decode::{
    AccumulatorTable, AluTable, Cc, Decode, DecodeCb, IndirectTable, R8, Rot, Rp, Rp2,
};
pub use interrupt::{Dispatches, InterruptRequested, InterruptVector, LowestRequest, Requested};
pub use machine::{Hl, Holds, InRom, Machine, PairValue};

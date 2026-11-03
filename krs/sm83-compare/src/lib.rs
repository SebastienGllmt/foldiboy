//! The comparator: a small KRS definition whose run compares two SM83
//! observations (draft-personal/observer-design/design.md, D3, D4).
//!
//! Its inputs are the two observations, the tier and the normalizations.
//! One rule replaces `#compare` with the verdict, `equal` or the first
//! difference, so `krs run --case` prints it in the computation cell.
//! Observations from any system reach it in the typed input encoding.

use k::builtin::{KSequence, List};
use sm83_observe::{Compare, Normalization, Observation, Tier};

k::sorts! {
    Command;
}

#[k::constructors(sort = Command, result = k::KResult)]
pub enum Step {
    /// Compare the two observations.
    #[k::syntax("#compare")]
    Compare,
}

#[k::configuration]
pub struct Comparison {
    #[k::cell]
    pub k: KSequence,
    #[k::cell]
    pub left: Observation,
    #[k::cell]
    pub right: Observation,
    #[k::cell]
    pub tier: Tier,
    #[k::cell]
    pub normalizations: List<Normalization>,
}

impl Comparison {
    #[k::init]
    pub fn init(
        #[k::config_var(name = "LEFT")] left: Observation,
        #[k::config_var(name = "RIGHT")] right: Observation,
        #[k::config_var(name = "TIER")] tier: Tier,
        #[k::config_var(name = "NORMALIZATIONS")] normalizations: List<Normalization>,
    ) -> Self {
        Self { k: k::k![Step::Compare], left, right, tier, normalizations }
    }
}

k::rules! {
    for Comparison;
    compare: {
        k: [Step::Compare, ..],
        left,
        right,
        tier,
        normalizations,
    } => {
        k: [Compare(left, right, tier, normalizations), ..],
    };
}

pub mod sequence;
pub mod abductive;
pub mod intent; // New
pub mod planning; // Placeholder for Phase 6

pub use sequence::SequenceResonator;
pub use abductive::AbductiveReasoner;
pub use intent::{IntentResolver, Intent};

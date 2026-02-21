#![deny(warnings)]
pub mod ooda;
pub mod adaptation;
pub mod scrubber;
pub mod profiler;

pub use ooda::{OODAController, Action, Goal};
pub use adaptation::{HardwareAdapter, PowerMode};
pub use scrubber::MemoryScrubber;
pub use profiler::{Profiler, ProfileScope};

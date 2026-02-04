#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OodaState { Observe, Orient, Decide, Act }
pub struct OodaLoop { pub current_state: OodaState }
impl OodaLoop {
    pub fn new() -> Self { Self { current_state: OodaState::Observe } }
    pub fn step(&mut self) {
        self.current_state = match self.current_state {
            OodaState::Observe => OodaState::Orient,
            OodaState::Orient => OodaState::Decide,
            OodaState::Decide => OodaState::Act,
            OodaState::Act => OodaState::Observe,
        };
    }
}

use crate::OmniMind;
use log::info;

pub struct LearningEngine;

impl LearningEngine {
    pub fn reward_usage(mind: &mut OmniMind, key: &str) {
        info!("Rewarding usage of fact: {}", key);
        match &mut mind.state {
            crate::LifecycleState::Forge(mem, _) => {
                if let Some(entry) = mem.metadata.get_mut(key) {
                    entry.reinforce(1.0);
                    info!("Reinforced in Forge: confidence={:.2}", entry.confidence);
                }
            },
            crate::LifecycleState::Runtime(base, delta, _) => {
                if let Some(entry) = delta.metadata.get_mut(key) {
                    entry.reinforce(1.0);
                } else if let Some(entry) = base.metadata.get_mut(key) {
                    entry.reinforce(1.0);
                }
            }
        }
    }

    pub fn penalize_usage(mind: &mut OmniMind, key: &str) {
        info!("Penalizing incorrect usage of fact: {}", key);
        match &mut mind.state {
            crate::LifecycleState::Forge(mem, _) => {
                if let Some(entry) = mem.metadata.get_mut(key) {
                    entry.confidence *= 0.8;
                    entry.importance *= 0.9;
                }
            },
            crate::LifecycleState::Runtime(_, delta, _) => {
                if let Some(entry) = delta.metadata.get_mut(key) {
                    entry.confidence *= 0.8;
                    entry.importance *= 0.9;
                }
            }
        }
    }
}

use crate::CognitionCore;
use std::collections::{HashSet, VecDeque};
use serde::{Deserialize, Serialize};

// Planning Engine & Goals

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub description: String,
    pub target_state: String,
    pub priority: u8,
    pub completed: bool,
}

impl CognitionCore {
    /// Add a new goal to the system.
    pub fn add_goal(&mut self, description: String, target_state: String, priority: u8) {
        self.goals.push(Goal {
            description,
            target_state,
            priority,
            completed: false,
        });
    }

    /// Mark a goal as completed.
    pub fn complete_goal(&mut self, target_state: &str) {
        for goal in &mut self.goals {
            if goal.target_state == target_state {
                goal.completed = true;
            }
        }
    }

    /// List active goals.
    pub fn active_goals(&self) -> Vec<&Goal> {
        self.goals.iter().filter(|g| !g.completed).collect()
    }

    /// Finds a path of actions/relations to get from start_state to end_state.
    /// This is a simplified symbolic planner (Action abstraction via Causal/Temporal links).
    pub fn find_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((start.to_string(), vec![start.to_string()]));
        visited.insert(start.to_string());

        let mut steps = 0;
        const MAX_PLAN_STEPS: usize = 200;

        while let Some((current, path)) = queue.pop_front() {
            steps += 1;
            if steps > MAX_PLAN_STEPS { return None; }

            if current == end {
                return Some(path);
            }

            // Limit depth for performance in prototype
            if path.len() > 10 { continue; }

            if let Some(relations) = self.relation_graph.get(&current) {
                for rel in relations {
                    // Planning traverses Causal or Generic links
                    if !visited.contains(&rel.target) {
                        visited.insert(rel.target.clone());
                        let mut new_path = path.clone();
                        new_path.push(rel.target.clone());
                        queue.push_back((rel.target.clone(), new_path));
                    }
                }
            }
        }
        None
    }
}

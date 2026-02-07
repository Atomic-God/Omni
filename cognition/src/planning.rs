use crate::CognitionCore;
use std::collections::{HashSet, VecDeque};

// Planning Engine
// Symbolic Planning using Graph Search

impl CognitionCore {
    /// Finds a path of actions/relations to get from start_state to end_state.
    /// This is a simplified symbolic planner (Action abstraction via Causal/Temporal links).
    pub fn find_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((start.to_string(), vec![start.to_string()]));
        visited.insert(start.to_string());

        while let Some((current, path)) = queue.pop_front() {
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

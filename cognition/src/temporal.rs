use crate::{CognitionCore, RelationType};
use std::collections::VecDeque;

impl CognitionCore {
    // === Temporal Reasoning ===

    /// Returns a list of events ordered by time (based on Temporal relations).
    /// Naive topological sort or chain traversal.
    pub fn order_events(&self, events: Vec<String>) -> Vec<String> {
        let mut ordered = Vec::new();
        // Simple strategy: check if A precedes B
        // For N events, this is O(N^2) checks against graph.

        // Better: Build a local graph of these events and topo-sort.
        // For MVP: Just sort by simple "A happens before B" check.

        let mut remaining: VecDeque<String> = events.into_iter().collect();

        // This is a naive sort, real temporal reasoning needs a full constraint solver.
        // But for "Timeline logic", this is a start.
        if !remaining.is_empty() {
            ordered.push(remaining.pop_front().unwrap());
        }

        while let Some(current) = remaining.pop_front() {
            let mut inserted = false;
            for i in 0..ordered.len() {
                if self.happens_before(&current, &ordered[i]) {
                    ordered.insert(i, current.clone());
                    inserted = true;
                    break;
                }
            }
            if !inserted {
                ordered.push(current);
            }
        }

        ordered
    }

    pub fn happens_before(&self, a: &str, b: &str) -> bool {
        // BFS for Temporal edge
        self.bfs_relation_check(a, b, RelationType::Temporal)
    }

    fn bfs_relation_check(&self, start: &str, target: &str, target_type: RelationType) -> bool {
        let mut queue = VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if current == target { return true; } // Should not happen if start != target initially, but catches identity

            if let Some(relations) = self.relation_graph.get(&current) {
                for rel in relations {
                    if rel.relation_type == target_type {
                        if rel.target == target { return true; }
                        if !visited.contains(&rel.target) {
                            visited.insert(rel.target.clone());
                            queue.push_back(rel.target.clone());
                        }
                    }
                }
            }
        }
        false
    }
}

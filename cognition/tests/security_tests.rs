use cognition::{CognitionCore, RelationType};

#[test]
fn test_basic_graph_ops() {
    let mut core = CognitionCore::new();
    core.add_relation("A", "B", RelationType::Causal, 1.0, 1.0);
    assert!(core.relation_graph.contains_key("A"));
}

use core_vsa::HyperVector;
use crate::{SemanticVector, Modality};
use cognition::CognitionCore;
use std::collections::HashMap;

pub struct MultimodalBridge;

impl MultimodalBridge {
    pub fn fuse_modalities(vectors: &[SemanticVector], dim: usize) -> SemanticVector {
        if vectors.is_empty() {
            return SemanticVector::new(HyperVector::deterministic_dim(0, dim), Modality::CrossModal, "Empty".to_string(), "Bridge");
        }

        let mut fused_vec = vectors[0].vector.clone();
        let mut labels = Vec::new();

        for sv in vectors.iter() {
            fused_vec = fused_vec.bundle(&sv.vector);
            labels.push(sv.label.clone());
        }

        SemanticVector::new(fused_vec, Modality::CrossModal, format!("Fused: [{}]", labels.join(" + ")), "Bridge")
    }

    /// cross-modal retrieval: Find text labels that match a visual vector.
    pub fn describe_image(
        visual_vec: &HyperVector,
        _cognition: &CognitionCore,
        memory: &HashMap<String, HyperVector>
    ) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        for (label, vec) in memory {
            // High similarity suggests the visual vector 'matches' this conceptual text vector
            let sim = visual_vec.similarity(vec);
            if sim > 0.3 {
                results.push((label.clone(), sim));
            }
        }

        // Also check Knowledge Graph for related entities
        // (Simplified for Phase 1)

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(3);
        results
    }

    /// cross-modal retrieval: Find similar sounds for a given concept.
    pub fn find_similar_sounds(
        concept_vec: &HyperVector,
        audio_memory: &[(String, HyperVector)]
    ) -> Vec<(String, f32)> {
        let mut results = Vec::new();
        for (id, vec) in audio_memory {
            let sim = concept_vec.similarity(vec);
            if sim > 0.2 {
                results.push((id.clone(), sim));
            }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Links modalities in the Knowledge Graph.
    pub fn link_modalities(
        core: &mut CognitionCore,
        entity_a: &str,
        entity_b: &str,
        relation: &str,
        confidence: f32
    ) {
        use cognition::RelationType;
        core.add_relation(entity_a, entity_b, RelationType::Taxonomic, 1.0, confidence);
        if relation == "represents" || relation == "describes" {
            core.add_relation(entity_b, entity_a, RelationType::Structural, 1.0, confidence);
        }
    }
}

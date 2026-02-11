use cognition::CognitionCore;
use cognition::traits::{PerceptionModule, ReasoningModule};
use log::{info, warn};
use memory::{EncoderConfig, MemoryStore, MindPack, VocabStore, LearningPolicies, MindMetadata, LifecycleState, PersonalMemory};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use learning::LearningEngine;
use ingestion::{SemanticChunk, ChunkMetadata};

pub mod context;
pub mod explanation;
pub mod security;
pub mod governance;
pub mod adapter; // Added
pub mod metrics; // Added
pub mod budget; // Added

use context::ContextManager;
use adapter::RuntimeAdapter;
use metrics::RuntimeMetrics;
use explanation::{ExplanationEngine, AudienceModel};
use security::PermissionBoundary;

/// Trait for extending capabilities.
pub trait ExtensionModule: Send + Sync {
    fn process(&self, input: &str, mind: &mut ForgeMind);
}

// ==================================================================================
// FORGE MIND (The Creator / Factory)
// Unlimited memory, Mutable, used for Fabrication.
// ==================================================================================

pub struct ForgeMind {
    pub cognition: CognitionCore,
    extensions: Vec<Box<dyn ExtensionModule>>,
    pub context: ContextManager,
    pub permissions: PermissionBoundary,
}

impl Default for ForgeMind {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgeMind {
    pub fn new() -> Self {
        info!("Initializing ForgeMind (Creator Instance).");
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        Self {
            cognition: CognitionCore::new(),
            extensions: Vec::new(),
            context: ContextManager::new(),
            permissions: PermissionBoundary::new(cwd),
        }
    }

    pub fn learn(&mut self, text: &str) {
        info!("Forge Learning: {}", text);
        self.cognition.learn_text(text);
        self.context.log_episodic(text);
        self.context.activate_semantic(text);

        // Extensions
        let exts = std::mem::take(&mut self.extensions);
        for ext in &exts {
            ext.process(text, self);
        }
        self.extensions = exts;
    }

    pub fn load_master(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.permissions.check_path(std::path::Path::new(path)).map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        info!("Loading Master Mind from {}", path);
        let pack = memory::load_snapshot(path)?;
        self.cognition = pack.memory.core;
        Ok(())
    }

    // Alias for compatibility
    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.load_master(path)
    }

    pub fn save_master(&self, path: &str) -> Result<(), std::io::Error> {
        let path_obj = std::path::Path::new(path);
        if let Some(parent) = path_obj.parent() {
             let check_path = if parent.as_os_str().is_empty() {
                 std::env::current_dir()?
             } else {
                 parent.to_path_buf()
             };
             self.permissions.check_path(&check_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;
        }

        info!("Saving Master Mind to {}", path);
        let pack = self.to_pack("master", LifecycleState::Fabricated);
        memory::save_snapshot(&pack, path)
    }

    // Alias for compatibility
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        self.save_master(path)
    }

    // Snapshot creation (Immutable Export)
    pub fn snapshot(&self, version: &str, source: &str, path: &str) -> Result<(), std::io::Error> {
        let path_obj = std::path::Path::new(path);
        if let Some(parent) = path_obj.parent() {
             let check_path = if parent.as_os_str().is_empty() {
                 std::env::current_dir()?
             } else {
                 parent.to_path_buf()
             };
             self.permissions.check_path(&check_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;
        }

        info!("Creating Immutable Snapshot v{} at {}", version, path);
        let pack = self.to_pack(source, LifecycleState::Frozen);
        memory::save_snapshot(&pack, path)
    }

    fn to_pack(&self, source: &str, state: LifecycleState) -> MindPack {
        let core_hash = self.cognition.compute_integrity_hash();
        MindPack {
            version: "8.4".to_string(), // Schema version
            memory: MemoryStore {
                core: self.cognition.clone(),
                relational_index: memory::index::RelationalIndex::new(), // Initialize
            },
            vocab: VocabStore { words: self.cognition.index_memory.clone() },
            encoder_config: EncoderConfig { model_name: "beagle-v5".to_string() },
            learning_policies: LearningPolicies {
                reinforcement_rate: 0.1,
                decay_rate: 0.01,
                max_concepts: None, // Unlimited for Forge
            },
            metadata: MindMetadata {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                core_hash,
                source: source.to_string(),
                state,
                compiler_version: env!("CARGO_PKG_VERSION").to_string(),
                semantic_version: "1.0.0".to_string(), // Default, should be arg
            },
            manifest: None, // Renamed from blueprint
        }
    }

    pub fn ask(&mut self, question: &str) -> String {
        self.context.log_episodic(question);
        self.context.activate_semantic(question);

        if question.starts_with("Explain ") {
            let concept = &question[8..];
            return ExplanationEngine::simplify(&self.cognition, concept, AudienceModel::Student);
        }

        self.cognition.query(question)
    }

    pub fn explain(&self, concept: &str) -> String {
        self.cognition.explain_concept(concept)
    }

    pub fn introspect(&self) -> String {
         format!("Forge Status: {} concepts, {} relations (Unlimited)\nContext: {:?}",
             self.cognition.index_memory.len(),
             self.cognition.relation_graph.len(),
             self.context.get_active_context())
    }
}

// ==================================================================================
// RUNTIME MIND (The Consumer / Sovereign)
// Base (Frozen) + Overlay (Mutable/Bounded).
// ==================================================================================

pub struct RuntimeMind {
    pub base: Arc<MindPack>,      // Read-Only Global Knowledge
    pub overlay: PersonalMemory,  // Read-Write Local Context
    pub context: ContextManager,
    pub learner: LearningEngine,  // Continuous Learning Engine
    pub adapter: RuntimeAdapter,  // Hardware adaptation
    pub metrics: RuntimeMetrics,  // Observability
}

impl RuntimeMind {
    pub fn load(base_path: &str, overlay_path: Option<&str>) -> Result<Self, std::io::Error> {
        info!("Loading RuntimeMind Base from {}", base_path);
        let base = memory::load_snapshot(base_path)?;

        let mut overlay = if let Some(ov_path) = overlay_path {
            if std::path::Path::new(ov_path).exists() {
                info!("Loading Personal Overlay from {}", ov_path);
                let ov = memory::load_personal(ov_path)?;
                if ov.parent_hash != base.metadata.core_hash {
                    warn!("Overlay hash mismatch! This overlay belongs to a different Mind version. Creating new overlay.");
                    Self::create_overlay(&base)
                } else {
                    ov
                }
            } else {
                info!("Creating new Personal Overlay at {}", ov_path);
                Self::create_overlay(&base)
            }
        } else {
            Self::create_overlay(&base)
        };

        let mut context = ContextManager::new();
        if !overlay.context_state.is_empty() {
            if let Ok(ctx) = bincode::serde::decode_from_slice::<ContextManager, _>(&overlay.context_state, bincode::config::standard()) {
                context = ctx.0;
            }
        }

        Ok(Self {
            base: Arc::new(base),
            overlay,
            context,
            learner: LearningEngine::new(),
            adapter: RuntimeAdapter::new(),
            metrics: RuntimeMetrics::new(),
        })
    }

    fn create_overlay(base: &MindPack) -> PersonalMemory {
        PersonalMemory {
            core: CognitionCore::new(),
            parent_hash: base.metadata.core_hash.clone(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            last_accessed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            context_state: Vec::new(),
        }
    }

    pub fn save_overlay(&self, path: &str) -> Result<(), std::io::Error> {
        // Serialize Context
        let mut overlay_copy = self.overlay.clone();
        if let Ok(bytes) = bincode::serde::encode_to_vec(&self.context, bincode::config::standard()) {
            overlay_copy.context_state = bytes;
        }
        memory::save_personal(&overlay_copy, path)
    }

    pub fn learn_personal(&mut self, text: &str) {
        self.metrics.learning_events += 1;
        info!("Runtime Personal Learning: {}", text);
        // Check budget before learning
        let concept_count = self.overlay.core.index_memory.len();
        if concept_count > 5000 { // Hardcoded runtime limit for now, or use policy
             warn!("Personal Memory Full ({} concepts). Triggering consolidation/pruning.", concept_count);
             // Let LearningEngine handle consolidation
        }

        // Create chunk
        let chunk = SemanticChunk {
            source: "user_input".to_string(),
            content: text.to_string(),
            metadata: ChunkMetadata {
                hash: format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                file_type: "text".to_string(),
                language: "en".to_string(),
                structure_type: "conversation".to_string(),
            }
        };

        // Use LearningEngine (handles entropy, consolidation)
        self.learner.learn(&mut self.overlay.core, vec![chunk]);

        // Update Context
        self.context.activate_semantic(text);
        self.context.log_episodic(text);
        self.overlay.last_accessed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    }

    pub fn ask(&mut self, question: &str) -> String {
        self.metrics.query_count += 1;
        self.context.activate_semantic(question);
        self.context.log_episodic(question);

        // 1. Try Overlay First
        let overlay_answer = self.overlay.core.query(question);
        if self.is_valid_answer(&overlay_answer) {
            return format!("(Personal) {}", overlay_answer);
        }

        // 2. Fallback to Base
        let base_answer = self.base.memory.core.query(question);
        if self.is_valid_answer(&base_answer) {
             return base_answer;
        }

        "I do not know.".to_string()
    }

    fn is_valid_answer(&self, ans: &str) -> bool {
        ans != "Unknown" && ans != "No connection found." && ans != "Query too short." && !ans.is_empty()
    }

    pub fn introspect(&self) -> String {
        format!("Runtime Status:\n- Base Concepts: {}\n- Personal Concepts: {}\n- Overlay Hash: {}\n- Context: {:?}",
            self.base.memory.core.index_memory.len(),
            self.overlay.core.index_memory.len(),
            self.overlay.parent_hash,
            self.context.get_active_context())
    }
}

// Legacy Alias for compatibility during refactor
pub type OmniMind = ForgeMind;

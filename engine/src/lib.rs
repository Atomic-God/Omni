use cognition::traits::{PerceptionModule, ReasoningModule};
use cognition::CognitionCore;
use cognition::planning::Goal;
use log::{error, info};
use memory::{EncoderConfig, MemoryStore, MindPack, VocabStore, LearningPolicies, MindMetadata, LifecycleState};
use perception::decoder::TextDecoder;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;

/// Trait for extending OmniMind capabilities.
pub trait ExtensionModule: Send + Sync {
    fn process(&self, input: &str, mind: &mut OmniMind);
}

/// OmniMind is the high-level interface for the Omni Forge system.
pub struct OmniMind {
    pub cognition: CognitionCore,
    extensions: Vec<Box<dyn ExtensionModule>>,
    pub read_only: bool,
    pub goals: VecDeque<Goal>,
}

impl Default for OmniMind {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniMind {
    pub fn new() -> Self {
        info!("Initializing OmniMind instance.");
        Self {
            cognition: CognitionCore::new(),
            extensions: Vec::new(),
            read_only: false,
            goals: VecDeque::new(),
        }
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        if self.read_only && !read_only {
             error!("Security Violation: Attempted to revert READ-ONLY mode.");
             panic!("Runtime Integrity Violation: Cannot revert from frozen state.");
        }
        self.read_only = read_only;
        if read_only {
            info!("OmniMind switched to READ-ONLY mode. Learning is permanently disabled.");
        }
    }

    pub fn register_extension(&mut self, extension: Box<dyn ExtensionModule>) {
        self.extensions.push(extension);
    }

    #[cfg(feature = "fabrication")]
    pub fn learn(&mut self, text: &str) {
        if self.read_only {
            error!("Security Violation: Attempted to learn in READ-ONLY mode.");
            panic!("Runtime Integrity Violation: Learning forbidden in runtime phase.");
        }

        info!("Learning text: {}", text);
        self.cognition.learn_text(text);

        let exts = std::mem::take(&mut self.extensions);
        for ext in &exts {
            ext.process(text, self);
        }
        self.extensions = exts;
    }

    #[cfg(not(feature = "fabrication"))]
    pub fn learn(&mut self, _text: &str) {
        error!("Security Violation: Fabrication features not compiled in.");
        panic!("Runtime Integrity Violation: This binary is compiled for Runtime only. Learning is impossible.");
    }

    pub fn ask(&self, question: &str) -> String {
        info!("Processing query: {}", question);
        let answer = self.cognition.query(question);

        if answer == "Unknown" || answer == "No connection found." {
            "I do not have enough information to answer that based on my current experiences.".to_string()
        } else {
            // Enhanced Output is now in cognition.query_internal
            answer
        }
    }

    pub fn explain(&self, concept: &str) -> String {
        info!("Processing explain request for: {}", concept);
        self.cognition.explain_concept(concept)
    }

    pub fn plan(&self, start: &str, end: &str) -> String {
        info!("Planning path from {} to {}", start, end);
        if let Some(path) = self.cognition.find_path(start, end) {
            format!("Plan found: {}", path.join(" -> "))
        } else {
            "No plan found.".to_string()
        }
    }

    pub fn add_goal(&mut self, description: &str, target: &str, priority: u8) {
        if self.read_only {
             // Goals might be allowed in runtime? Or are they "mutations"?
             // Goals are internal state, not "learning". Let's allow it for now as "Runtime State".
             // But OmniMind structs are usually reloaded.
             // If we want persistent goals, we need to save them.
             // For now, allow in memory.
        }
        self.goals.push_back(Goal {
            description: description.to_string(),
            target_state: target.to_string(),
            priority,
            completed: false,
        });
        // Sort by priority (descending)
        let mut vec: Vec<Goal> = self.goals.drain(..).collect();
        vec.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.goals = VecDeque::from(vec);
    }

    pub fn generate(&self, hv: &core_vsa::HyperVector) -> String {
        let decoder = TextDecoder::new(self.cognition.semantic_memory.clone());
        decoder.decode_svo(hv).0
    }

    pub fn introspect(&self) -> String {
        let memory_size = self.cognition.index_memory.len();
        let relations = self.cognition.relation_graph.values().map(|v| v.len()).sum::<usize>();
        let active_goals = self.goals.len();

        format!(
            "Mind Status:\n- Concepts: {}\n- Relations: {}\n- Active Goals: {}\n- Mode: {}",
            memory_size, relations, active_goals,
            if self.read_only { "READ-ONLY" } else { "LEARNING" }
        )
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        info!("Saving mind to {}", path);

        // Use proper integrity hash from CognitionCore
        let core_hash = self.cognition.compute_integrity_hash();

        let pack = MindPack {
            version: "8.0".to_string(),
            memory: MemoryStore {
                core: self.cognition.clone(),
            },
            vocab: VocabStore {
                words: self.cognition.index_memory.clone(),
            },
            encoder_config: EncoderConfig {
                model_name: "beagle-v5".to_string(),
            },
            learning_policies: LearningPolicies {
                reinforcement_rate: 0.1,
                decay_rate: 0.01,
            },
            metadata: MindMetadata {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                core_hash,
                source: "OmniForge v8.0 Fabricator".to_string(),
                state: if self.read_only { LifecycleState::Frozen } else { LifecycleState::Fabricated },
                compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        match memory::save_mind(&pack, path) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to save mind: {}", e);
                Err(e)
            }
        }
    }

    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        info!("Loading mind from {}", path);
        match memory::load_mind(path) {
            Ok(pack) => {
                // Check Lifecycle
                match pack.metadata.state {
                    LifecycleState::Fabricated | LifecycleState::Frozen | LifecycleState::Runtime => {
                        self.cognition = pack.memory.core;
                        self.read_only = true; // Enforce runtime read-only
                        info!("Mind loaded successfully in READ-ONLY mode. State: {:?}", pack.metadata.state);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                error!("Failed to load mind: {}", e);
                Err(e)
            }
        }
    }
}

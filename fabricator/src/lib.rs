use engine::ForgeMind;
use memory::{MindPack, MemoryStore, VocabStore, EncoderConfig, LearningPolicies, MindMetadata, LifecycleState, MindBlueprint};
use log::{info, warn};
use std::path::PathBuf;
use learning::LearningEngine;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod facade;
pub mod live;
pub mod merge;
pub mod compiler;
pub mod packaging; // Added

use compiler::{ForgeCompiler, TargetProfile};
use packaging::MindPackagingPipeline;

pub struct ForgeConfig {
    pub max_concepts: Option<usize>,
    pub reinforcement_rate: f32,
    pub decay_rate: f32,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            max_concepts: None,
            reinforcement_rate: 0.1,
            decay_rate: 0.01,
        }
    }
}

pub struct ForgeSession {
    pub mind: ForgeMind,
    pub config: ForgeConfig,
    pub learning_engine: LearningEngine,
}

impl ForgeSession {
    pub fn new(config: ForgeConfig) -> Self {
        Self {
            mind: ForgeMind::new(),
            config,
            learning_engine: LearningEngine::new(),
        }
    }

    pub fn absorb(&mut self, data_path: &str) {
        info!("Forge Session Absorbing: {}", data_path);
        let path = PathBuf::from(data_path);
        if path.exists() {
             let chunks = ingestion::ingest_path(path);
             self.learning_engine.learn(&mut self.mind.cognition, chunks);
             self.learning_engine.consolidate(&mut self.mind.cognition);
        } else {
            warn!("Absorb path invalid: {}", data_path);
        }
    }

    pub fn finalize_snapshot(&self, version: &str, blueprint: Option<MindBlueprint>) -> MindPack {
        let core_hash = self.mind.cognition.compute_integrity_hash();
        MindPack {
            version: "8.3".to_string(),
            memory: MemoryStore { core: self.mind.cognition.clone() },
            vocab: VocabStore { words: self.mind.cognition.index_memory.clone() },
            encoder_config: EncoderConfig { model_name: "beagle-v5".to_string() },
            learning_policies: LearningPolicies {
                reinforcement_rate: self.config.reinforcement_rate,
                decay_rate: self.config.decay_rate,
                max_concepts: self.config.max_concepts,
            },
            metadata: MindMetadata {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                core_hash,
                source: "Forge Session".to_string(),
                state: LifecycleState::Frozen,
                compiler_version: env!("CARGO_PKG_VERSION").to_string(),
                semantic_version: version.to_string(),
            },
            blueprint,
        }
    }

    pub fn compile(&self, profile_name: &str, output_path: &str) -> Result<(), std::io::Error> {
        let profile = TargetProfile {
            name: profile_name.to_string(),
            max_memory_mb: 1024, // Default
            quantization: false,
        };
        ForgeCompiler::compile_target(&self.mind, profile, "1.0.0-compiled", output_path)
    }

    pub fn pack(&self, target: &str, output_path: &str) -> Result<(), std::io::Error> {
        let profile = match target {
            "mobile" => packaging::TargetProfile { name: "mobile".to_string(), max_memory_mb: 512, quantization: true },
            "server" => packaging::TargetProfile { name: "server".to_string(), max_memory_mb: 16384, quantization: false },
            _ => packaging::TargetProfile { name: "desktop".to_string(), max_memory_mb: 4096, quantization: false },
        };
        MindPackagingPipeline::pack(&self.mind, profile, "1.0.0-packed", output_path)
    }
}

pub struct FabricationPipeline;

impl FabricationPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn fabricate(&self, data_path: &str) -> MindPack {
        info!("Starting fabrication from: {}", data_path);
        let mut session = ForgeSession::new(ForgeConfig::default());
        session.absorb(data_path);
        info!("Fabrication complete. Packaging mind.");
        session.finalize_snapshot("1.0.0", None)
    }
}

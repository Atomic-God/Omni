use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProfile {
    pub max_memory_mb: usize,
    pub simd_width: usize, // 0 = scalar, 128 = neon/sse, 256 = avx2, 512 = avx512
    pub has_gpu: bool,
    pub concurrency_limit: usize,
}

impl Default for CapabilityProfile {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            simd_width: 0,
            has_gpu: false,
            concurrency_limit: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPolicy {
    LowPower,
    Balanced,
    HighPerformance,
}

pub struct RuntimeAdapter {
    pub capabilities: CapabilityProfile,
    pub policy: ExecutionPolicy,
}

impl RuntimeAdapter {
    pub fn new() -> Self {
        let hte_profile = hte::detect();

        let simd_width = if hte_profile.avx512 { 512 }
        else if hte_profile.avx2 { 256 }
        else if hte_profile.neon { 128 }
        else { 0 };

        let capabilities = CapabilityProfile {
            max_memory_mb: (hte_profile.total_memory / 1024 / 1024) as usize,
            simd_width,
            has_gpu: false, // CPU-only baseline for now
            concurrency_limit: hte_profile.logical_cores,
        };

        Self {
            capabilities,
            policy: ExecutionPolicy::Balanced,
        }
    }

    pub fn configure_policy(&mut self, policy: ExecutionPolicy) {
        self.policy = policy;
    }

    pub fn suggest_batch_size(&self) -> usize {
        match self.policy {
            ExecutionPolicy::LowPower => 1,
            ExecutionPolicy::Balanced => 16,
            ExecutionPolicy::HighPerformance => 64 * (self.capabilities.concurrency_limit.max(1)),
        }
    }
}

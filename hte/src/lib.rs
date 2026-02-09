pub mod dispatch;
pub mod isa;
pub mod topology;
pub mod os;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::CpuId;
use sysinfo::{System, SystemExt};
use log::warn;

#[derive(Debug, Default, Clone)]
pub struct HardwareProfile {
    // ISA Extensions (Execution Speed Only)
    pub avx2: bool,
    pub avx512_f: bool,
    pub avx512_bw: bool,
    pub neon: bool,
    pub sve: bool,
    pub amx: bool,
    pub ane: bool,

    // Topology (Scheduling Only)
    pub l1_cache_size: Option<usize>,
    pub l2_cache_size: Option<usize>,
    pub l3_cache_size: Option<usize>,
    pub physical_cores: usize,
    pub logical_cores: usize,

    // Runtime Awareness (Safety Only)
    pub total_memory: u64,
    pub used_memory: u64,
    pub memory_budget_margin: f32,
    pub thermal_throttled: bool,
    pub os_name: String,
    pub kernel_version: String,
}

pub fn detect() -> HardwareProfile {
    let mut profile = HardwareProfile::default();
    let mut sys = System::new_all();
    sys.refresh_all();

    profile.total_memory = sys.total_memory();
    profile.used_memory = sys.used_memory();

    let usage_ratio = profile.used_memory as f32 / profile.total_memory as f32;
    profile.memory_budget_margin = (1.0 - usage_ratio).max(0.0);

    profile.os_name = sys.name().unwrap_or("Unknown".to_string());
    profile.kernel_version = sys.kernel_version().unwrap_or("Unknown".to_string());
    profile.thermal_throttled = false;

    profile.update_isa();
    profile.update_topology();

    if profile.memory_budget_margin < 0.1 {
        warn!("HTE Alert: Critical Memory Pressure (Margin < 10%). Suggesting reduced recursion.");
    }

    profile
}

impl HardwareProfile {
    // SECURITY LOCK: This function verifies that HTE is read-only for reasoning.
    pub fn verify_execution_only(&self) {
        // No-op assertion that compiles away, but serves as a contract.
        // If we added "learning_rate_modifier" here, this would be the place to reject it.
    }

    fn update_isa(&mut self) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let cpuid = CpuId::new();
            if let Some(info) = cpuid.get_extended_feature_info() {
                self.avx2 = info.has_avx2();
                self.avx512_f = info.has_avx512f();
                self.avx512_bw = info.has_avx512bw();
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            self.neon = true;
            self.sve = false;
            self.amx = false;
            self.ane = false;
        }
    }

    fn update_topology(&mut self) {
        self.logical_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        self.physical_cores = self.logical_cores;
        let (l1, l2, l3) = topology::detect_cache_sizes();
        self.l1_cache_size = l1;
        self.l2_cache_size = l2;
        self.l3_cache_size = l3;
    }
}

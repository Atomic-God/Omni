pub mod dispatch;
pub mod isa;
pub mod topology;
pub mod os; // Added

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::CpuId;
use sysinfo::{System, SystemExt};
use log::warn;

#[derive(Debug, Default, Clone)]
pub struct HardwareProfile {
    // ISA Extensions
    pub avx2: bool,
    pub avx512_f: bool,
    pub avx512_bw: bool,
    pub neon: bool,
    pub sve: bool,
    pub amx: bool, // Apple Matrix Coprocessor
    pub ane: bool, // Apple Neural Engine

    // Topology / Cache (sizes in bytes)
    pub l1_cache_size: Option<usize>,
    pub l2_cache_size: Option<usize>,
    pub l3_cache_size: Option<usize>,
    pub physical_cores: usize,
    pub logical_cores: usize,

    // Runtime Awareness
    pub total_memory: u64, // KB
    pub used_memory: u64, // KB
    pub memory_budget_margin: f32, // 0.0 - 1.0 (safety margin)
    pub thermal_throttled: bool, // If CPU is thermally limited
    pub os_name: String,
    pub kernel_version: String,
}

pub fn detect() -> HardwareProfile {
    let mut profile = HardwareProfile::default();

    // System Info
    let mut sys = System::new_all();
    sys.refresh_all();

    profile.total_memory = sys.total_memory();
    profile.used_memory = sys.used_memory();

    // Safety Margin: If > 90% used, margin is low.
    let usage_ratio = profile.used_memory as f32 / profile.total_memory as f32;
    profile.memory_budget_margin = (1.0 - usage_ratio).max(0.0);

    profile.os_name = sys.name().unwrap_or("Unknown".to_string());
    profile.kernel_version = sys.kernel_version().unwrap_or("Unknown".to_string());

    // CPU Info
    // Check if any CPU core frequency is dropping drastically (mock heuristic for throttling)
    // Or check explicit thermal files if on Linux
    // For now, assume false unless we detect high load and low freq?
    // Let's just expose basic "No" for now as sysinfo doesn't robustly report thermal throttling across all OS.
    profile.thermal_throttled = false;

    // 1. ISA Detection
    profile.update_isa();

    // 2. Topology Detection
    profile.update_topology();

    if profile.memory_budget_margin < 0.1 {
        warn!("HTE Alert: Critical Memory Pressure (Margin < 10%). Suggesting reduced recursion.");
    }

    profile
}

impl HardwareProfile {
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
            // In a real Android/iOS env, we'd query getauxval or sysctl
            // For now, assume NEON is present on AArch64
            self.neon = true;

            // SVE detection placeholder (requires specific kernel flags)
            self.sve = false;

            // Apple Silicon checks would go here (e.g. sysctl hw.optional.arm.FEAT_AMX)
            self.amx = false;
            self.ane = false;
        }
    }

    fn update_topology(&mut self) {
        self.logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        self.physical_cores = self.logical_cores; // Fallback logic

        // Cache detection logic
        let (l1, l2, l3) = topology::detect_cache_sizes();
        self.l1_cache_size = l1;
        self.l2_cache_size = l2;
        self.l3_cache_size = l3;
    }
}

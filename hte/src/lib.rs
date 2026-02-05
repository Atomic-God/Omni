pub mod dispatch;
pub mod isa;
pub mod topology;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::CpuId;

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
}

pub fn detect() -> HardwareProfile {
    let mut profile = HardwareProfile::default();

    // 1. ISA Detection
    profile.update_isa();

    // 2. Topology Detection
    profile.update_topology();

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

        let (l1, l2, l3) = topology::detect_cache_sizes();
        self.l1_cache_size = l1;
        self.l2_cache_size = l2;
        self.l3_cache_size = l3;
    }
}

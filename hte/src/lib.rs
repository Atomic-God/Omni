#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::CpuId;

#[derive(Debug, Default)]
pub struct HardwareProfile {
    pub avx2: bool,
    pub avx512: bool,
    pub neon: bool,
}

pub fn detect() -> HardwareProfile {
    let mut profile = HardwareProfile::default();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cpuid = CpuId::new();
        if let Some(info) = cpuid.get_extended_feature_info() {
             profile.avx2 = info.has_avx2();
             profile.avx512 = info.has_avx512f();
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        profile.neon = true;
    }
    profile
}

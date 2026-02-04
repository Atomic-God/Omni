#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use raw_cpuid::CpuId;

pub fn detect_cache_sizes() -> (Option<usize>, Option<usize>, Option<usize>) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cpuid = CpuId::new();
        let mut l1 = None;
        let mut l2 = None;
        let mut l3 = None;

        if let Some(cparams) = cpuid.get_cache_parameters() {
            for cache in cparams {
                let size = cache.sets() * cache.associativity() * cache.coherency_line_size();
                match cache.level() {
                    1 => l1 = Some(size),
                    2 => l2 = Some(size),
                    3 => l3 = Some(size),
                    _ => {}
                }
            }
        }
        (l1, l2, l3)
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        // Placeholder for non-x86 topology detection
        (None, None, None)
    }
}

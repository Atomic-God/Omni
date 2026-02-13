use raw_cpuid::CpuId;
use sysinfo::{System, SystemExt, CpuExt};
use log::info;
use std::time::Instant;

pub mod isa;
pub mod topology;
pub mod dispatch;

pub struct HardwareProfile {
    pub logical_cores: usize,
    pub physical_cores: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub avx2: bool,
    pub avx512: bool,
    pub neon: bool,
    pub amx: bool,
    pub memory_bandwidth_mbps: f64, // Estimated
}

pub fn detect() -> HardwareProfile {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpuid = CpuId::new();
    let avx2 = cpuid.get_feature_info().map_or(false, |f| f.has_avx2());
    let avx512 = cpuid.get_feature_info().map_or(false, |f| f.has_avx()); // Simplified check

    // Bandwidth Benchmark
    let bandwidth = benchmark_memory();

    HardwareProfile {
        logical_cores: sys.cpus().len(),
        physical_cores: sys.physical_core_count().unwrap_or(1),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        avx2,
        avx512,
        neon: false, // Need ARM check via std::arch or file reading
        amx: false,
        memory_bandwidth_mbps: bandwidth,
    }
}

fn benchmark_memory() -> f64 {
    // Allocate 100MB
    let size = 100 * 1024 * 1024;
    let mut data = vec![0u8; size];
    let start = Instant::now();

    // Write
    for i in 0..size {
        data[i] = (i % 255) as u8;
    }
    // Read
    let mut sum: u64 = 0;
    for i in 0..size {
        sum += data[i] as u64;
    }

    let duration = start.elapsed().as_secs_f64();
    let mb = (size as f64 * 2.0) / 1024.0 / 1024.0; // Read + Write
    mb / duration
}

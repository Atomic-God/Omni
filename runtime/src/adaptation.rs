use hte::{HardwareProfile, detect, get_current_load};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    LowPower,
    Balanced,
    HighPerformance,
    Mobile, // Aggressive low-power mode
}

pub struct HardwareAdapter {
    pub profile: HardwareProfile,
    pub current_mode: PowerMode,
    pub mobile_mode_forced: bool,
}

impl HardwareAdapter {
    pub fn new() -> Self {
        let profile = detect();
        let load = get_current_load();
        let mode = if load > 80.0 {
            PowerMode::LowPower
        } else if load > 40.0 {
            PowerMode::Balanced
        } else {
            PowerMode::HighPerformance
        };

        Self {
            profile,
            current_mode: mode,
            mobile_mode_forced: false,
        }
    }

    pub fn set_mode(&mut self, mode: PowerMode) {
        if mode != self.current_mode {
            info!("Industrial Adaptation: Switching to PowerMode: {:?}", mode);
            self.current_mode = mode;
        }
    }

    /// Dynamically adjusts the power mode based on live telemetry (thermal/load/RAM).
    pub fn live_adjust(&mut self) {
        // Refresh live RAM stats for dynamic scaling
        let fresh = detect();
        self.profile.used_memory = fresh.used_memory;
        self.profile.available_memory = fresh.available_memory;

        let load = get_current_load();
        let thermal = self.profile.thermal_limit;
        let avail_ram_gb = self.profile.available_memory / 1024 / 1024 / 1024;

        let new_mode = if load > 85.0 || thermal > 80.0 || avail_ram_gb < 1 {
            PowerMode::LowPower
        } else if self.is_mobile_platform() {
            PowerMode::Mobile
        } else if load < 30.0 && avail_ram_gb > 8 {
            PowerMode::HighPerformance
        } else {
            PowerMode::Balanced
        };

        self.set_mode(new_mode);
    }

    fn is_mobile_platform(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "android", target_os = "ios"))] {
                true
            } else {
                false
            }
        }
    }

    pub fn get_concurrency_limit(&self) -> usize {
        let base = match self.current_mode {
            PowerMode::LowPower | PowerMode::Mobile => 1,
            PowerMode::Balanced => (self.profile.physical_cores / 2).max(1),
            PowerMode::HighPerformance => self.profile.logical_cores,
        };

        let load = get_current_load();
        if load > 95.0 { 1 } else { base }
    }

    pub fn suggest_dimension(&self) -> usize {
        let avail_ram_gb = self.profile.available_memory / 1024 / 1024 / 1024;

        // RAM-aware base dimension scaling
        let base = if avail_ram_gb < 1 {
            1024 // Extreme low memory
        } else if avail_ram_gb < 2 {
            2048
        } else if avail_ram_gb < 4 {
            5120
        } else if avail_ram_gb < 12 {
            10000
        } else {
            20000
        };

        match self.current_mode {
            PowerMode::LowPower => (base / 2).max(1024),
            PowerMode::Balanced => base,
            PowerMode::HighPerformance => (base * 2).min(32000), // Max industrial precision
            PowerMode::Mobile => 2048, // Balanced for mobile
        }
    }

    /// Toggles aggressive low-power features for mobile environments.
    pub fn get_mobile_tuning(&self) -> MobileTuning {
        if self.current_mode == PowerMode::Mobile || self.mobile_mode_forced {
            MobileTuning {
                disable_background_tasks: true,
                reduce_io_frequency: true,
                max_concurrency: 1,
                use_fast_hashes_only: true,
            }
        } else {
            MobileTuning::default()
        }
    }

    pub fn is_low_memory_mode(&self) -> bool {
        let avail_ram_gb = self.profile.available_memory / 1024 / 1024 / 1024;
        avail_ram_gb < 1 || self.current_mode == PowerMode::LowPower
    }

    pub fn report(&self) {
        info!("Industrial Hardware Report [Mode: {:?}]:", self.current_mode);
        info!("  Load: {:.1}%", get_current_load());
        info!("  Suggested VSA Dim: {}", self.suggest_dimension());
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MobileTuning {
    pub disable_background_tasks: bool,
    pub reduce_io_frequency: bool,
    pub max_concurrency: usize,
    pub use_fast_hashes_only: bool,
}

impl Default for MobileTuning {
    fn default() -> Self {
        Self {
            disable_background_tasks: false,
            reduce_io_frequency: false,
            max_concurrency: 4,
            use_fast_hashes_only: false,
        }
    }
}

pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Tracks and logs live resource utilization with detailed industrial metrics.
    pub fn track_telemetry() {
        let load = get_current_load();
        let profile = detect();
        let used_mem_gb = profile.used_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let total_mem_gb = profile.total_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let mem_usage_pct = (used_mem_gb / total_mem_gb) * 100.0;

        info!("Industrial Telemetry: [CPU Load: {:.1}%] [RAM Used: {:.2}/{:.2} GB ({:.1}%)]",
            load, used_mem_gb, total_mem_gb, mem_usage_pct);

        if load > 90.0 {
            warn!("Industrial Alert: CPU Load exceeding industrial safety limits!");
        }
        if mem_usage_pct > 85.0 {
            warn!("Industrial Alert: RAM usage critically high ({:.1}%)!", mem_usage_pct);
        }
    }

    /// Captures a point-in-time performance snapshot.
    pub fn capture_snapshot() -> serde_json::Value {
        let load = get_current_load();
        let profile = detect();
        serde_json::json!({
            "cpu_load": load,
            "ram_used_bytes": profile.used_memory,
            "ram_total_bytes": profile.total_memory,
            "thermal_limit": profile.thermal_limit,
            "throttling": profile.throttling_active,
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        })
    }
}

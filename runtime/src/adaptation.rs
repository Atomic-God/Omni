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

    /// Dynamically adjusts the power mode based on live telemetry (thermal/load).
    pub fn live_adjust(&mut self) {
        let load = get_current_load();
        let thermal = self.profile.thermal_limit;

        let new_mode = if load > 85.0 || thermal > 80.0 {
            PowerMode::LowPower
        } else if load < 30.0 {
            PowerMode::HighPerformance
        } else {
            PowerMode::Balanced
        };

        self.set_mode(new_mode);
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
        if self.current_mode == PowerMode::Mobile || self.mobile_mode_forced {
            return 2048; // Strictly enforce low precision for mobile
        }

        let total_ram_gb = self.profile.total_memory / 1024 / 1024 / 1024;

        let base = if total_ram_gb < 2 {
            2048
        } else if total_ram_gb < 8 {
            10000
        } else {
            20000
        };

        match self.current_mode {
            PowerMode::LowPower => base / 5,
            _ => base,
        }
    }

    pub fn is_low_memory_mode(&self) -> bool {
        let free_ram_gb = (self.profile.total_memory - self.profile.used_memory) / 1024 / 1024 / 1024;
        free_ram_gb < 1 || self.current_mode == PowerMode::LowPower
    }

    pub fn report(&self) {
        info!("Industrial Hardware Report [Mode: {:?}]:", self.current_mode);
        info!("  Load: {:.1}%", get_current_load());
        info!("  Suggested VSA Dim: {}", self.suggest_dimension());
    }
}

pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Tracks and logs live resource utilization.
    pub fn track_telemetry() {
        let load = get_current_load();
        let profile = detect();
        let used_mem_gb = profile.used_memory as f32 / 1024.0 / 1024.0 / 1024.0;

        info!("Industrial Telemetry: [CPU Load: {:.1}%] [RAM Used: {:.2} GB]", load, used_mem_gb);

        if load > 90.0 {
            warn!("Industrial Alert: CPU Load exceeding industrial safety limits!");
        }
    }
}

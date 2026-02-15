use hte::{HardwareProfile, detect, get_current_load};
use log::{info, warn};

pub struct HardwareAdapter {
    pub profile: HardwareProfile,
}

impl HardwareAdapter {
    pub fn new() -> Self {
        Self {
            profile: detect(),
        }
    }

    pub fn suggest_dimension(&self) -> usize {
        let total_ram_gb = self.profile.total_memory / 1024 / 1024 / 1024;

        if total_ram_gb < 2 {
            info!("Low memory detected ({}GB). Suggesting 2,048-bit dimension.", total_ram_gb);
            2048
        } else if total_ram_gb < 8 {
            info!("Standard memory detected ({}GB). Suggesting 10,000-bit dimension.", total_ram_gb);
            10000
        } else {
            info!("High memory detected ({}GB). Suggesting 20,000-bit dimension.", total_ram_gb);
            20000
        }
    }

    pub fn suggest_batch_size(&self) -> usize {
        let load = get_current_load();
        if load > 90.0 {
            warn!("System load very high ({:.1}%). Reducing batch size to 1.", load);
            1
        } else if load > 60.0 {
            4
        } else {
            16
        }
    }

    pub fn is_low_memory_mode(&self) -> bool {
        let free_ram_gb = (self.profile.total_memory - self.profile.used_memory) / 1024 / 1024 / 1024;
        free_ram_gb < 1
    }

    pub fn report(&self) {
        info!("Hardware Adaptation Report:");
        info!("  Cores: {}/{}", self.profile.physical_cores, self.profile.logical_cores);
        info!("  RAM: {}MB / {}MB", self.profile.used_memory / 1024 / 1024, self.profile.total_memory / 1024 / 1024);
        info!("  AVX512: {}, AVX2: {}, NEON: {}", self.profile.avx512, self.profile.avx2, self.profile.neon);
        info!("  Current Load: {:.1}%", get_current_load());
    }
}

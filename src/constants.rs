//! Constants for the MS-DOS Defrag Simulator
//! 
//! These constants are designed to be reusable across different defrag simulations
//! (MS-DOS, Windows 95, Windows 98, etc.)

/// Disk drive configuration constants
pub mod disk {
    /// Represents a disk drive configuration with IOPS-based audio speed
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DriveConfig {
        pub letter: char,
        pub capacity_mb: u32,
        pub cluster_count: u32,
        pub iops: u32,
    }

    /// Drive C: Standard hard disk (slower, 2 IOPS)
    pub const DRIVE_C: DriveConfig = DriveConfig {
        letter: 'C',
        capacity_mb: 2048,
        cluster_count: 4096,
        iops: 2,
    };

    /// Drive D: Medium hard disk (3 IOPS)
    pub const DRIVE_D: DriveConfig = DriveConfig {
        letter: 'D',
        capacity_mb: 1024,
        cluster_count: 2048,
        iops: 3,
    };

    /// Drive E: Slow disk/Floppy (1 IOPS - slowest)
    pub const DRIVE_E: DriveConfig = DriveConfig {
        letter: 'E',
        capacity_mb: 512,
        cluster_count: 1024,
        iops: 1,
    };

    /// Drive F: Fast SSHD/SSD hybrid (8 IOPS - fastest)
    pub const DRIVE_F: DriveConfig = DriveConfig {
        letter: 'F',
        capacity_mb: 2048,
        cluster_count: 4096,
        iops: 8,
    };

    /// All available drive configurations
    pub const ALL_DRIVES: [DriveConfig; 4] = [DRIVE_C, DRIVE_D, DRIVE_E, DRIVE_F];

    /// Default drive (Drive C)
    pub const DEFAULT_DRIVE: DriveConfig = DRIVE_C;

    /// Get drive by letter
    pub fn get_drive_by_letter(letter: char) -> Option<DriveConfig> {
        ALL_DRIVES.iter().find(|d| d.letter == letter).copied()
    }

    /// Get drive by index
    pub fn get_drive_by_index(index: usize) -> Option<DriveConfig> {
        ALL_DRIVES.get(index).copied()
    }
}

/// Audio configuration constants
pub mod audio {
    /// Minimum playback rate (for very low IOPS)
    pub const MIN_PLAYBACK_RATE: f32 = 0.5;
    
    /// Maximum playback rate (for very high IOPS)
    pub const MAX_PLAYBACK_RATE: f32 = 4.0;
    
    /// IOPS range for mapping [min, max]
    pub const IOPS_RANGE: (u32, u32) = (0, 16);
    
    /// Playback rate range for mapping [min, max]
    pub const RATE_RANGE: (f32, f32) = (MIN_PLAYBACK_RATE, MAX_PLAYBACK_RATE);
    
    /// Default audio volume (0.0 to 1.0)
    pub const DEFAULT_VOLUME: f32 = 0.5;

    /// Calculates playback rate based on IOPS using linear mapping
    /// 
    /// This maps IOPS values from [0, 16] to playback rates [0.5, 4.0]
    /// Higher IOPS = faster playback rate (more realistic disk sound simulation)
    /// 
    /// # Arguments
    /// * `iops` - Input/Output Operations Per Second of the disk drive
    /// 
    /// # Returns
    /// Playback rate between MIN_PLAYBACK_RATE and MAX_PLAYBACK_RATE
    /// 
    /// # Example
    /// ```
    /// use defrag_rs::constants::audio::calculate_playback_rate;
    /// 
    /// // Slow disk (1 IOPS) -> slow playback
    /// let rate = calculate_playback_rate(1);
    /// assert!(rate < 1.0);
    /// 
    /// // Fast disk (8 IOPS) -> faster playback  
    /// let rate = calculate_playback_rate(8);
    /// assert!(rate > 1.0);
    /// ```
    pub fn calculate_playback_rate(iops: u32) -> f32 {
        let (iops_min, iops_max) = IOPS_RANGE;
        let (rate_min, rate_max) = RATE_RANGE;
        

        let iops_f = iops as f32;
        let iops_min_f = iops_min as f32;
        let iops_max_f = iops_max as f32;
        
        let rate = (iops_f - iops_min_f) * (rate_max - rate_min) / (iops_max_f - iops_min_f) + rate_min;
        
        // Clamp to valid range
        rate.max(MIN_PLAYBACK_RATE).min(MAX_PLAYBACK_RATE)
    }

    /// Alternative calculation based on simple IOPS timing (1000ms / iops)
    /// This gives a more dramatic difference between slow and fast disks
    pub fn calculate_playback_rate_timing(iops: u32) -> f32 {
        if iops == 0 {
            return MIN_PLAYBACK_RATE;
        }
        
        // Higher IOPS = lower delay = faster playback
        // Base timing: 1000ms / iops, then normalize
        let timing = 1000.0 / (iops as f32);
        
        // Normalize: slower timing (higher value) = slower playback
        // We want IOPS 1 -> slow, IOPS 8 -> fast
        // So we invert: faster timing = higher rate
        let normalized = 1000.0 / timing; // This equals iops
        
        // Scale to our range
        let rate = normalized / 4.0; // Scale factor to get reasonable rates
        
        rate.max(MIN_PLAYBACK_RATE).min(MAX_PLAYBACK_RATE)
    }
}

/// Animation timing constants
pub mod animation {
    /// Default tick rate in milliseconds
    pub const DEFAULT_TICK_RATE_MS: u64 = 80;
    
    /// Fast animation tick rate
    pub const FAST_TICK_RATE_MS: u64 = 40;
    
    /// Slow animation tick rate  
    pub const SLOW_TICK_RATE_MS: u64 = 150;
    
    /// Initialization phase duration (in ticks)
    pub const INIT_DURATION_TICKS: u64 = 20;
    
    /// Finished phase wait time (in ticks) before auto-exit
    pub const FINISH_WAIT_TICKS: u64 = 50;
}

/// UI dimensions and layout constants
pub mod ui {
    /// Default grid width
    pub const DEFAULT_GRID_WIDTH: usize = 78;
    
    /// Default grid height
    pub const DEFAULT_GRID_HEIGHT: usize = 16;
    
    /// Default disk fill percentage
    pub const DEFAULT_FILL_PERCENT: f32 = 0.65;
    
    /// Percentage of bad blocks
    pub const BAD_BLOCK_PERCENT: f32 = 0.02;
    
    /// Progress bar width in characters
    pub const PROGRESS_BAR_WIDTH: usize = 38;
    
    /// About box width
    pub const ABOUT_BOX_WIDTH: u16 = 52;
    
    /// About box height
    pub const ABOUT_BOX_HEIGHT: u16 = 18;
}

/// Defrag simulation types
pub mod defrag_type {
    /// Different defrag visual styles
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum DefragStyle {
        /// MS-DOS 6.x style defrag (text-based)
        MsDos,
        /// Windows 95 style defrag (graphical)
        Windows95,
        /// Windows 98 style defrag (graphical, improved)
        Windows98,
    }

    impl DefragStyle {
        /// Returns the display name of the defrag style
        pub fn name(&self) -> &'static str {
            match self {
                DefragStyle::MsDos => "MS-DOS 6.x Defrag",
                DefragStyle::Windows95 => "Windows 95 Defrag",
                DefragStyle::Windows98 => "Windows 98 Defrag",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_rate_calculation() {
        // IOPS 0 should give minimum rate
        assert_eq!(audio::calculate_playback_rate(0), audio::MIN_PLAYBACK_RATE);
        
        // IOPS 16 should give maximum rate
        assert_eq!(audio::calculate_playback_rate(16), audio::MAX_PLAYBACK_RATE);
        
        // IOPS 8 should be in the middle
        let rate = audio::calculate_playback_rate(8);
        assert!(rate > audio::MIN_PLAYBACK_RATE && rate < audio::MAX_PLAYBACK_RATE);
    }

    #[test]
    fn test_drive_lookup() {
        assert_eq!(disk::get_drive_by_letter('C'), Some(disk::DRIVE_C));
        assert_eq!(disk::get_drive_by_letter('F'), Some(disk::DRIVE_F));
        assert_eq!(disk::get_drive_by_letter('Z'), None);
    }
}

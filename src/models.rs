use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClusterState {
    Used,      // Already defragmented block (green)
    Unused,    // Free block
    Pending,   // Block to be defragmented (white)
    Bad,       // Bad block
    Unmovable, // Unmovable system block
    Reading,   // Block being read
    Writing,   // Block being written
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DefragPhase {
    Initializing,
    Analyzing,
    Defragmenting,
    Finished,
}

#[derive(Clone)]
pub struct DefragStats {
    pub total_to_defrag: usize,    // Total number of clusters to defragment
    pub clusters_defragged: usize, // Number of defragmented clusters
    pub start_time: Instant,
}

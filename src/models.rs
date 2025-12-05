use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClusterState {
    Used,       // Bloc déjà défragmenté (vert)
    Unused,     // Bloc libre
    Pending,    // Bloc à défragmenter (blanc)
    Bad,        // Bloc défectueux
    Unmovable,  // Bloc système non déplaçable
    Reading,    // Bloc en cours de lecture
    Writing,    // Bloc en cours d'écriture
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
    pub total_to_defrag: usize,  // Nombre total de clusters à défragmenter
    pub clusters_defragged: usize, // Nombre de clusters défragmentés
    pub start_time: Instant,
}
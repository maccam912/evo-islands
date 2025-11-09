use crate::Genome;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client -> Server: Request work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRequest {
    /// Client ID (persistent across sessions)
    pub client_id: Uuid,

    /// Protocol version the client is using
    pub protocol_version: u32,

    /// Client version string
    pub client_version: String,
}

/// A genome paired with its lineage ID for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeWithId {
    pub genome_id: Uuid,
    pub genome: Genome,
}

/// Server -> Client: Work assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAssignment {
    /// Unique ID for this work unit
    pub work_id: Uuid,

    /// Seed genomes to start the island with (with IDs for lineage tracking)
    /// Version 2: includes genome IDs for survival tracking
    pub seed_genomes_v2: Vec<GenomeWithId>,

    /// Grid width for spatial simulation
    pub grid_width: usize,

    /// Grid height for spatial simulation
    pub grid_height: usize,

    /// Maximum simulation steps
    pub max_steps: u32,

    /// Mutation rate (0.0 to 1.0)
    pub mutation_rate: f64,

    // Legacy fields for backwards compatibility (deprecated)
    #[serde(default)]
    pub seed_genomes: Vec<Genome>,

    #[serde(default)]
    pub generations: u32,

    #[serde(default)]
    pub population_size: usize,
}

/// Survival statistics for a genome lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurvivalResult {
    pub genome_id: Uuid,
    pub survived: u32,
    pub total_spawned: u32,
    pub avg_lifespan: f64,
    pub total_food_eaten: u32,
}

/// Client -> Server: Work result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResult {
    /// The work ID this is responding to
    pub work_id: Uuid,

    /// Client ID
    pub client_id: Uuid,

    /// Survival results for each genome (Version 2)
    #[serde(default)]
    pub survival_results: Vec<SurvivalResult>,

    /// Number of simulation steps completed
    pub steps_completed: u32,

    // Legacy fields for backwards compatibility (deprecated)
    #[serde(default)]
    pub best_genomes: Vec<GenomeWithFitness>,

    #[serde(default)]
    pub generations_completed: u32,

    #[serde(default)]
    pub stats: Option<SimulationStats>,
}

/// A genome paired with its fitness score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeWithFitness {
    pub genome: Genome,
    pub fitness: f64,
}

/// Statistics about a simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStats {
    /// Average fitness over all generations
    pub avg_fitness: f64,

    /// Best fitness achieved
    pub best_fitness: f64,

    /// Final population size
    pub final_population: usize,

    /// Total creatures that lived during simulation
    pub total_creatures: usize,
}

/// Server -> Client: Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    /// Client protocol version doesn't match server
    VersionMismatch {
        server_version: u32,
        client_version: u32,
    },

    /// Server is overloaded, try again later
    ServerOverloaded,

    /// Invalid request
    InvalidRequest(String),

    /// Internal server error
    InternalError(String),
}

/// Stats about the global evolution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    /// Total number of clients connected
    pub active_clients: usize,

    /// Total work units completed
    pub total_work_units: u64,

    /// Total generations simulated across all clients
    pub total_generations: u64,

    /// Current best genomes
    pub best_genomes: Vec<GenomeWithFitness>,

    /// Size of the gene pool
    pub gene_pool_size: usize,

    /// Server uptime in seconds
    pub uptime_seconds: u64,
}

impl WorkRequest {
    pub fn new(client_id: Uuid, protocol_version: u32) -> Self {
        Self {
            client_id,
            protocol_version,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl WorkAssignment {
    /// Create a new spatial simulation work assignment (Version 2)
    pub fn new_spatial(
        seed_genomes_v2: Vec<GenomeWithId>,
        grid_width: usize,
        grid_height: usize,
        max_steps: u32,
        mutation_rate: f64,
    ) -> Self {
        Self {
            work_id: Uuid::new_v4(),
            seed_genomes_v2,
            grid_width,
            grid_height,
            max_steps,
            mutation_rate,
            // Legacy fields
            seed_genomes: vec![],
            generations: 0,
            population_size: 0,
        }
    }

    /// Create a legacy work assignment (Version 1 - deprecated)
    pub fn new(
        seed_genomes: Vec<Genome>,
        generations: u32,
        population_size: usize,
        mutation_rate: f64,
    ) -> Self {
        Self {
            work_id: Uuid::new_v4(),
            seed_genomes_v2: vec![],
            grid_width: 0,
            grid_height: 0,
            max_steps: 0,
            mutation_rate,
            seed_genomes,
            generations,
            population_size,
        }
    }
}

impl SimulationStats {
    pub fn new(
        avg_fitness: f64,
        best_fitness: f64,
        final_population: usize,
        total_creatures: usize,
    ) -> Self {
        Self {
            avg_fitness,
            best_fitness,
            final_population,
            total_creatures,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_request_serialization() {
        let req = WorkRequest::new(Uuid::new_v4(), 1);
        let json = serde_json::to_string(&req).unwrap();
        let decoded: WorkRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.client_id, decoded.client_id);
    }

    #[test]
    fn test_work_assignment_serialization() {
        let assignment = WorkAssignment::new(vec![Genome::random()], 100, 50, 0.05);
        let json = serde_json::to_string(&assignment).unwrap();
        let decoded: WorkAssignment = serde_json::from_str(&json).unwrap();
        assert_eq!(assignment.work_id, decoded.work_id);
    }
}

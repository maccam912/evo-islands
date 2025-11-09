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

/// Server -> Client: Work assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAssignment {
    /// Unique ID for this work unit
    pub work_id: Uuid,

    /// Seed genomes to start the island with
    /// These include the best known genomes plus some random historical ones
    pub seed_genomes: Vec<Genome>,

    /// How many generations to simulate
    pub generations: u32,

    /// Population size for the island
    pub population_size: usize,

    /// Mutation rate (0.0 to 1.0)
    pub mutation_rate: f64,
}

/// Client -> Server: Work result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResult {
    /// The work ID this is responding to
    pub work_id: Uuid,

    /// Client ID
    pub client_id: Uuid,

    /// Best genomes discovered during simulation
    pub best_genomes: Vec<GenomeWithFitness>,

    /// Number of generations actually simulated
    pub generations_completed: u32,

    /// Statistics from the simulation
    pub stats: SimulationStats,
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
    pub fn new(
        seed_genomes: Vec<Genome>,
        generations: u32,
        population_size: usize,
        mutation_rate: f64,
    ) -> Self {
        Self {
            work_id: Uuid::new_v4(),
            seed_genomes,
            generations,
            population_size,
            mutation_rate,
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

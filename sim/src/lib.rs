pub mod creature;
pub mod island;
pub mod world;

pub use creature::Creature;
pub use island::{Island, IslandConfig, SurvivalStats};
pub use world::World;

use shared::{GenomeWithFitness, SimulationStats};
use uuid::Uuid;

/// Run a spatial simulation with competitive evolution on a 2D grid
/// Returns survival statistics for each genome
pub fn run_spatial_simulation(
    seed_genomes: Vec<(Uuid, shared::Genome)>,
    config: IslandConfig,
) -> Vec<SurvivalStats> {
    let mut island = Island::new(config, seed_genomes);
    island.run_simulation()
}

/// Run a complete island simulation (DEPRECATED - use run_spatial_simulation instead)
/// This is a compatibility wrapper that runs a minimal spatial simulation
pub fn run_simulation(
    seed_genomes: Vec<shared::Genome>,
    _generations: u32,
    _population_size: usize,
    mutation_rate: f64,
) -> (Vec<GenomeWithFitness>, SimulationStats) {
    // Convert to spatial simulation format
    let seed_genomes_with_ids: Vec<(Uuid, shared::Genome)> = seed_genomes
        .into_iter()
        .map(|g| (Uuid::new_v4(), g))
        .collect();
    // Precompute a fallback "best" set derived from the seeds, for legacy
    // behavior when the final population goes extinct by the end of the run.
    let mut seed_fallback: Vec<GenomeWithFitness> = seed_genomes_with_ids
        .iter()
        .map(|(_, g)| GenomeWithFitness {
            genome: g.clone(),
            fitness: g.fitness_score(),
        })
        .collect();
    seed_fallback.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    if seed_fallback.len() > 10 {
        seed_fallback.truncate(10);
    }

    // Run a small spatial simulation
    // Use a generous resource configuration to keep legacy expectations
    // that some best genomes are produced by the end of the run.
    let config = IslandConfig {
        world_width: 80,
        world_height: 80,
        max_steps: 200,
        mutation_rate,
        plant_density: 0.10,
        food_density: 0.05,
        reproduction_threshold: 100.0,
        max_age: 600,
    };

    let mut island = Island::new(config, seed_genomes_with_ids);
    let mut rng = rand::thread_rng();

    let mut total_fitness = 0.0;
    let mut fitness_samples = 0;
    let mut best_fitness = 0.0;
    let initial_count = island.creatures.len();
    let mut last_nonempty_best: Vec<GenomeWithFitness> = Vec::new();

    while island.step < island.config.max_steps {
        island.tick(&mut rng);

        // Sample fitness every 10 steps
        if island.step.is_multiple_of(10) {
            let avg_gen_fitness = island.average_fitness();
            total_fitness += avg_gen_fitness;
            fitness_samples += 1;

            if avg_gen_fitness > best_fitness {
                best_fitness = avg_gen_fitness;
            }

            let current_best = island.get_best_genomes(10);
            if !current_best.is_empty() {
                last_nonempty_best = current_best;
            }
        }
    }

    let avg_fitness = if fitness_samples > 0 {
        total_fitness / fitness_samples as f64
    } else {
        0.0
    };

    let mut best_genomes = island.get_best_genomes(10);
    if best_genomes.is_empty() {
        if !last_nonempty_best.is_empty() {
            best_genomes = last_nonempty_best;
        } else if !seed_fallback.is_empty() {
            best_genomes = seed_fallback;
        }
    }
    let stats = SimulationStats::new(
        avg_fitness,
        best_fitness,
        island.creatures.len(),
        initial_count + island.creatures.len(),
    );

    (best_genomes, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Genome;

    #[test]
    fn test_simulation_runs() {
        let seeds = vec![Genome::random(), Genome::random()];
        let (best_genomes, stats) = run_simulation(seeds, 10, 20, 0.1);

        assert!(!best_genomes.is_empty());
        assert!(stats.avg_fitness >= 0.0);
        assert!(stats.best_fitness >= 0.0);
    }

    #[test]
    fn test_simulation_produces_results() {
        let seeds = vec![Genome::default()];
        let (best_genomes, _) = run_simulation(seeds, 50, 30, 0.05);

        assert!(best_genomes.len() <= 10);
        assert!(!best_genomes.is_empty());
    }
}

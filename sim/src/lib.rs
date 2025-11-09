pub mod creature;
pub mod island;

pub use creature::Creature;
pub use island::{Island, IslandConfig};

use shared::{GenomeWithFitness, SimulationStats};

/// Run a complete island simulation
pub fn run_simulation(
    seed_genomes: Vec<shared::Genome>,
    generations: u32,
    population_size: usize,
    mutation_rate: f64,
) -> (Vec<GenomeWithFitness>, SimulationStats) {
    let config = IslandConfig {
        population_size,
        mutation_rate,
        food_per_tick: population_size as f64 * 0.8, // Slight scarcity
        reproduction_threshold: 100.0,
        max_age: 500,
    };

    let mut island = Island::new(config, seed_genomes);

    let mut total_fitness = 0.0;
    let mut fitness_samples = 0;
    let mut best_fitness = 0.0;
    let mut total_creatures = island.creatures.len();

    for _ in 0..generations {
        island.tick();

        // Sample fitness
        let avg_gen_fitness = island.average_fitness();
        total_fitness += avg_gen_fitness;
        fitness_samples += 1;

        if avg_gen_fitness > best_fitness {
            best_fitness = avg_gen_fitness;
        }

        total_creatures += island.creatures.len();
    }

    let avg_fitness = if fitness_samples > 0 {
        total_fitness / fitness_samples as f64
    } else {
        0.0
    };

    let best_genomes = island.get_best_genomes(10);
    let stats = SimulationStats::new(
        avg_fitness,
        best_fitness,
        island.creatures.len(),
        total_creatures,
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

use crate::Creature;
use rand::seq::SliceRandom;
use shared::{Genome, GenomeWithFitness};

#[derive(Debug, Clone)]
pub struct IslandConfig {
    pub population_size: usize,
    pub mutation_rate: f64,
    pub food_per_tick: f64,
    pub reproduction_threshold: f64,
    pub max_age: u32,
}

pub struct Island {
    pub config: IslandConfig,
    pub creatures: Vec<Creature>,
    pub generation: u32,
}

impl Island {
    /// Create a new island with seed genomes
    pub fn new(config: IslandConfig, seed_genomes: Vec<Genome>) -> Self {
        let mut creatures = Vec::new();

        // Create initial population from seed genomes
        for _ in 0..config.population_size {
            let genome = if seed_genomes.is_empty() {
                Genome::random()
            } else {
                seed_genomes
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone()
            };
            creatures.push(Creature::new(genome));
        }

        Self {
            config,
            creatures,
            generation: 0,
        }
    }

    /// Advance the simulation by one tick
    pub fn tick(&mut self) {
        // 1. All creatures consume energy
        for creature in &mut self.creatures {
            creature.consume_energy();
        }

        // 2. Distribute food based on combat power (competition)
        self.distribute_food();

        // 3. Remove dead creatures and old ones
        self.creatures
            .retain(|c| !c.is_dead() && c.age < self.config.max_age);

        // 4. Reproduction phase
        self.reproduce();

        // 5. Maintain population size (add random creatures if too few)
        while self.creatures.len() < self.config.population_size / 4 {
            self.creatures.push(Creature::new(Genome::random()));
        }

        self.generation += 1;
    }

    /// Distribute food with competition
    fn distribute_food(&mut self) {
        if self.creatures.is_empty() {
            return;
        }

        // Calculate total combat power
        let total_power: f64 = self.creatures.iter().map(|c| c.combat_power()).sum();

        if total_power <= 0.0 {
            // Equal distribution if no one has power
            let food_per_creature = self.config.food_per_tick / self.creatures.len() as f64;
            for creature in &mut self.creatures {
                creature.add_energy(food_per_creature);
            }
        } else {
            // Distribute based on combat power (stronger creatures get more)
            for creature in &mut self.creatures {
                let power_ratio = creature.combat_power() / total_power;
                let food = self.config.food_per_tick * power_ratio;
                creature.add_energy(food);
            }
        }
    }

    /// Handle reproduction
    fn reproduce(&mut self) {
        let mut new_creatures = Vec::new();
        let mut rng = rand::thread_rng();

        // Need at least 2 creatures to reproduce
        if self.creatures.len() < 2 {
            return;
        }

        // Shuffle to randomize mating pairs
        let mut indices: Vec<usize> = (0..self.creatures.len()).collect();
        indices.shuffle(&mut rng);

        // Try to pair up creatures for reproduction
        for i in (0..indices.len() - 1).step_by(2) {
            let idx1 = indices[i];
            let idx2 = indices[i + 1];

            // Check if both can reproduce
            if self.creatures[idx1].can_reproduce(self.config.reproduction_threshold)
                && self.creatures[idx2].can_reproduce(self.config.reproduction_threshold)
            {
                // Use split_at_mut to get two mutable references safely
                let (left, right) = if idx1 < idx2 {
                    let (left, right) = self.creatures.split_at_mut(idx2);
                    (&mut left[idx1], &mut right[0])
                } else {
                    let (left, right) = self.creatures.split_at_mut(idx1);
                    (&mut right[0], &mut left[idx2])
                };

                // Create offspring
                if let Some(child) = left.reproduce(right, self.config.mutation_rate) {
                    new_creatures.push(child);
                }
            }
        }

        // Add new creatures
        self.creatures.extend(new_creatures);
    }

    /// Get average fitness of the population
    pub fn average_fitness(&self) -> f64 {
        if self.creatures.is_empty() {
            return 0.0;
        }

        let total: f64 = self.creatures.iter().map(|c| c.fitness()).sum();
        total / self.creatures.len() as f64
    }

    /// Get the best N genomes from the island
    pub fn get_best_genomes(&self, n: usize) -> Vec<GenomeWithFitness> {
        let mut creatures = self.creatures.clone();
        creatures.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap());

        creatures
            .iter()
            .take(n)
            .map(|c| GenomeWithFitness {
                genome: c.genome.clone(),
                fitness: c.fitness(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_island_creation() {
        let config = IslandConfig {
            population_size: 50,
            mutation_rate: 0.1,
            food_per_tick: 40.0,
            reproduction_threshold: 100.0,
            max_age: 500,
        };

        let seeds = vec![Genome::random()];
        let island = Island::new(config, seeds);

        assert_eq!(island.creatures.len(), 50);
        assert_eq!(island.generation, 0);
    }

    #[test]
    fn test_island_tick() {
        let config = IslandConfig {
            population_size: 20,
            mutation_rate: 0.1,
            food_per_tick: 20.0,
            reproduction_threshold: 100.0,
            max_age: 500,
        };

        let seeds = vec![Genome::random()];
        let mut island = Island::new(config, seeds);

        let initial_gen = island.generation;
        island.tick();

        assert_eq!(island.generation, initial_gen + 1);
    }

    #[test]
    fn test_average_fitness() {
        let config = IslandConfig {
            population_size: 10,
            mutation_rate: 0.1,
            food_per_tick: 10.0,
            reproduction_threshold: 100.0,
            max_age: 500,
        };

        let seeds = vec![Genome::default()];
        let island = Island::new(config, seeds);

        let fitness = island.average_fitness();
        assert!(fitness > 0.0);
    }

    #[test]
    fn test_get_best_genomes() {
        let config = IslandConfig {
            population_size: 20,
            mutation_rate: 0.1,
            food_per_tick: 20.0,
            reproduction_threshold: 100.0,
            max_age: 500,
        };

        let seeds = vec![Genome::random(), Genome::random()];
        let island = Island::new(config, seeds);

        let best = island.get_best_genomes(5);
        assert!(best.len() <= 5);
        assert!(!best.is_empty());

        // Should be sorted by fitness
        if best.len() > 1 {
            assert!(best[0].fitness >= best[1].fitness);
        }
    }
}

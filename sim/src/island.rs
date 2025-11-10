use crate::{Creature, World};
use rand::seq::SliceRandom;
use rand::Rng;
use shared::{Genome, GenomeWithFitness};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct IslandConfig {
    pub world_width: usize,
    pub world_height: usize,
    pub max_steps: u32,
    pub mutation_rate: f64,
    pub plant_density: f64,
    pub food_density: f64,
    pub reproduction_threshold: f64,
}

impl Default for IslandConfig {
    fn default() -> Self {
        Self {
            world_width: 300,
            world_height: 300,
            max_steps: 3000,
            mutation_rate: 0.05,
            plant_density: 0.08, // Increased from 5% to 8% for more food availability
            food_density: 0.04,  // Increased from 2% to 4% for more food availability
            reproduction_threshold: 60.0, // Reduced from 100.0 to match creature.rs changes
        }
    }
}

/// Results from a spatial simulation
#[derive(Debug, Clone)]
pub struct SurvivalStats {
    pub genome_id: Uuid,
    pub survived: u32,
    pub total_spawned: u32,
    pub total_food_eaten: u32,
}

pub struct Island {
    pub config: IslandConfig,
    pub world: World,
    pub creatures: Vec<Creature>,
    pub step: u32,
    genome_stats: HashMap<Uuid, GenomeLineage>,
}

#[derive(Debug, Clone)]
struct GenomeLineage {
    total_spawned: u32,
    total_food_eaten: u32,
}

impl Island {
    /// Create a new spatial island with seed genomes
    pub fn new(config: IslandConfig, seed_genomes: Vec<(Uuid, Genome)>) -> Self {
        let mut world = World::new(config.world_width, config.world_height);
        let mut rng = rand::thread_rng();

        // Initialize resources
        world.initialize_resources(&mut rng, config.plant_density, config.food_density);

        let mut creatures = Vec::new();
        let mut genome_stats = HashMap::new();

        // Create creatures from seed genomes
        for (genome_id, genome) in seed_genomes {
            // Random position
            let x = rng.gen_range(0..config.world_width);
            let y = rng.gen_range(0..config.world_height);

            creatures.push(Creature::new(genome, genome_id, x, y));

            // Initialize stats tracking
            genome_stats.insert(
                genome_id,
                GenomeLineage {
                    total_spawned: 1,
                    total_food_eaten: 0,
                },
            );
        }

        Self {
            config,
            world,
            creatures,
            step: 0,
            genome_stats,
        }
    }

    /// Run the complete spatial simulation
    pub fn run_simulation(&mut self) -> Vec<SurvivalStats> {
        let mut rng = rand::thread_rng();

        while self.step < self.config.max_steps && !self.should_stop() {
            self.tick(&mut rng);
        }

        self.collect_survival_stats()
    }

    /// Check if simulation should stop (only one genome type left)
    fn should_stop(&self) -> bool {
        let unique_genomes: std::collections::HashSet<Uuid> =
            self.creatures.iter().map(|c| c.genome_id).collect();
        unique_genomes.len() <= 1
    }

    /// Advance the simulation by one step
    pub fn tick<R: Rng>(&mut self, rng: &mut R) {
        // 1. Regrow plants
        self.world.tick_plants();

        // 2. Creatures sense and decide actions
        let actions = self.decide_actions(rng);

        // 3. Execute movements
        self.execute_movements(actions, rng);

        // 4. Creatures try to eat
        self.execute_eating(rng);

        // 5. All creatures consume energy and age
        for creature in &mut self.creatures {
            creature.consume_energy();
        }

        // 6. Remove dead creatures and update stats
        let dead_creatures: Vec<_> = self
            .creatures
            .iter()
            .filter(|c| c.is_dead())
            .cloned()
            .collect();

        for dead in dead_creatures {
            if let Some(stats) = self.genome_stats.get_mut(&dead.genome_id) {
                stats.total_food_eaten += dead.food_eaten;
            }
        }

        self.creatures.retain(|c| !c.is_dead());

        // 7. Reproduction phase
        self.reproduce(rng);

        self.step += 1;
    }

    /// Creatures sense environment and decide what to do
    fn decide_actions<R: Rng>(&self, rng: &mut R) -> Vec<(usize, Action)> {
        let mut actions = Vec::new();

        for (idx, creature) in self.creatures.iter().enumerate() {
            // Find food within vision radius
            let food_in_vision =
                self.world
                    .find_food_in_radius(creature.x, creature.y, creature.vision_radius());

            if let Some((food_x, food_y, _)) = food_in_vision.first() {
                // Move towards nearest food
                let direction = creature.direction_to(*food_x, *food_y);
                actions.push((idx, Action::Move(direction)));
            } else {
                // Random movement
                let directions = [
                    crate::creature::Direction::North,
                    crate::creature::Direction::South,
                    crate::creature::Direction::East,
                    crate::creature::Direction::West,
                    crate::creature::Direction::NorthEast,
                    crate::creature::Direction::NorthWest,
                    crate::creature::Direction::SouthEast,
                    crate::creature::Direction::SouthWest,
                ];
                let direction = directions.choose(rng).unwrap();
                actions.push((idx, Action::Move(*direction)));
            }
        }

        actions
    }

    /// Execute movement actions
    fn execute_movements<R: Rng>(&mut self, actions: Vec<(usize, Action)>, rng: &mut R) {
        for (idx, action) in actions {
            if idx >= self.creatures.len() {
                continue;
            }

            let Action::Move(direction) = action;
            let creature = &self.creatures[idx];
            if let Some((new_x, new_y)) = creature.try_move(
                direction,
                self.config.world_width,
                self.config.world_height,
                rng,
            ) {
                self.creatures[idx].x = new_x;
                self.creatures[idx].y = new_y;
            }
        }
    }

    /// Creatures try to eat food at their positions
    /// Implements hybrid combat: peaceful movement, but fight over food
    fn execute_eating<R: Rng>(&mut self, rng: &mut R) {
        // Group creatures by position
        let mut positions: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        for (idx, creature) in self.creatures.iter().enumerate() {
            positions
                .entry((creature.x, creature.y))
                .or_default()
                .push(idx);
        }

        // Process each position with creatures
        for ((x, y), creature_indices) in positions {
            let available_food = self.world.get_available_food(x, y);

            if available_food == 0 {
                continue; // No food here
            }

            if creature_indices.len() == 1 {
                // Single creature eats peacefully
                let idx = creature_indices[0];
                let food_eaten = self.world.consume_food(x, y, 10);
                self.creatures[idx].add_energy(food_eaten as f64);
                self.creatures[idx].food_eaten += food_eaten;
            } else {
                // Multiple creatures - COMBAT!
                self.resolve_combat(&creature_indices, x, y, rng);
            }
        }
    }

    /// Resolve combat between creatures at the same food source
    fn resolve_combat<R: Rng>(
        &mut self,
        creature_indices: &[usize],
        x: usize,
        y: usize,
        _rng: &mut R,
    ) {
        // Calculate combat powers
        let mut combatants: Vec<(usize, f64)> = creature_indices
            .iter()
            .map(|&idx| (idx, self.creatures[idx].combat_power()))
            .collect();

        // Sort by combat power (highest first)
        combatants.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Winner gets the food
        let (winner_idx, winner_power) = combatants[0];

        let food_eaten = self.world.consume_food(x, y, 10);
        self.creatures[winner_idx].add_energy(food_eaten as f64);
        self.creatures[winner_idx].food_eaten += food_eaten;

        // Losers take damage to health (25% of winner's combat power)
        for (loser_idx, _) in combatants.iter().skip(1) {
            self.creatures[*loser_idx].take_damage(winner_power * 0.25);
        }
    }

    /// Handle reproduction with population control
    fn reproduce<R: Rng>(&mut self, rng: &mut R) {
        let mut new_creatures = Vec::new();

        // Need at least 2 creatures to reproduce
        if self.creatures.len() < 2 {
            return;
        }

        // Calculate population limit: half of total world area
        let world_area = self.config.world_width * self.config.world_height;
        let population_limit = world_area / 2;

        // Shuffle to randomize mating pairs
        let mut indices: Vec<usize> = (0..self.creatures.len()).collect();
        indices.shuffle(rng);

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
                    // Check if we're at population limit
                    if self.creatures.len() + new_creatures.len() >= population_limit {
                        // Find creature with lowest health and zero energy to remove
                        if let Some(remove_idx) = self.find_weakest_creature() {
                            // Track stats before removal
                            let removed = self.creatures.remove(remove_idx);
                            if let Some(stats) = self.genome_stats.get_mut(&removed.genome_id) {
                                stats.total_food_eaten += removed.food_eaten;
                            }
                        } else {
                            // No creature with zero energy found, block spawning
                            continue;
                        }
                    }

                    // Track lineage
                    if let Some(stats) = self.genome_stats.get_mut(&child.genome_id) {
                        stats.total_spawned += 1;
                    }
                    new_creatures.push(child);
                }
            }
        }

        // Add new creatures
        self.creatures.extend(new_creatures);
    }

    /// Find the creature with lowest health and zero energy
    /// Returns None if no creature has zero energy
    fn find_weakest_creature(&self) -> Option<usize> {
        let mut weakest_idx: Option<usize> = None;
        let mut lowest_health = f64::MAX;

        for (idx, creature) in self.creatures.iter().enumerate() {
            if creature.energy <= 0.0 {
                if weakest_idx.is_none() || creature.health < lowest_health {
                    weakest_idx = Some(idx);
                    lowest_health = creature.health;
                }
            }
        }

        weakest_idx
    }

    /// Collect survival statistics for all genomes
    fn collect_survival_stats(&self) -> Vec<SurvivalStats> {
        let mut results = Vec::new();

        // Count current survivors by genome
        let mut survivors: HashMap<Uuid, u32> = HashMap::new();
        for creature in &self.creatures {
            *survivors.entry(creature.genome_id).or_insert(0) += 1;
        }

        // Create stats for each genome we tracked
        for (genome_id, lineage) in &self.genome_stats {
            let survived = *survivors.get(genome_id).unwrap_or(&0);

            results.push(SurvivalStats {
                genome_id: *genome_id,
                survived,
                total_spawned: lineage.total_spawned,
                total_food_eaten: lineage.total_food_eaten,
            });
        }

        results
    }

    /// Get average fitness of the population (deprecated)
    pub fn average_fitness(&self) -> f64 {
        if self.creatures.is_empty() {
            return 0.0;
        }

        let total: f64 = self.creatures.iter().map(|c| c.fitness()).sum();
        total / self.creatures.len() as f64
    }

    /// Get the best N genomes from the island (deprecated - use survival stats instead)
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

#[derive(Debug, Clone, Copy)]
enum Action {
    Move(crate::creature::Direction),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_island_creation() {
        let config = IslandConfig::default();
        let genome_id = Uuid::new_v4();
        let seeds = vec![
            (genome_id, Genome::random()),
            (Uuid::new_v4(), Genome::random()),
        ];
        let island = Island::new(config, seeds);

        assert_eq!(island.creatures.len(), 2);
        assert_eq!(island.step, 0);
        assert_eq!(island.world.width, 300);
        assert_eq!(island.world.height, 300);
    }

    #[test]
    fn test_spatial_island_tick() {
        let config = IslandConfig {
            max_steps: 10,
            ..Default::default()
        };

        let genome_id = Uuid::new_v4();
        let seeds = vec![
            (genome_id, Genome::random()),
            (Uuid::new_v4(), Genome::random()),
        ];
        let mut island = Island::new(config, seeds);
        let mut rng = rand::thread_rng();

        island.tick(&mut rng);

        assert_eq!(island.step, 1);
    }

    #[test]
    fn test_simulation_runs() {
        let config = IslandConfig {
            world_width: 50,
            world_height: 50,
            max_steps: 100,
            plant_density: 0.1,
            food_density: 0.05,
            ..Default::default()
        };

        let genome_id = Uuid::new_v4();
        let seeds = vec![
            (genome_id, Genome::random()),
            (Uuid::new_v4(), Genome::random()),
        ];
        let mut island = Island::new(config, seeds);

        let results = island.run_simulation();

        assert!(!results.is_empty());
        assert!(island.step <= 100);
    }

    #[test]
    fn test_survival_stats_tracking() {
        let config = IslandConfig {
            world_width: 50,
            world_height: 50,
            max_steps: 50,
            ..Default::default()
        };

        let genome_id = Uuid::new_v4();
        let genome = Genome {
            efficiency: 1.0,
            ..Default::default()
        }; // High efficiency for survival

        let seeds = vec![(genome_id, genome)];
        let mut island = Island::new(config, seeds);

        let results = island.run_simulation();

        assert_eq!(results.len(), 1);
        assert!(results[0].total_spawned > 0);
    }
}

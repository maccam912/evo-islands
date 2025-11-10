use rand::Rng;
use shared::Genome;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Creature {
    pub id: Uuid,
    pub genome: Genome,
    pub genome_id: Uuid, // Original genome ID for lineage tracking
    pub energy: f64,
    pub health: f64,
    pub x: usize,
    pub y: usize,
    pub food_eaten: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Creature {
    /// Create a new creature with the given genome at a position
    pub fn new(genome: Genome, genome_id: Uuid, x: usize, y: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            genome,
            genome_id,
            energy: 100.0, // Starting energy
            health: 100.0, // Starting health
            x,
            y,
            food_eaten: 0,
        }
    }

    /// Consume energy based on genome
    pub fn consume_energy(&mut self) {
        // DISABLED: Creatures no longer lose energy naturally
        // Energy only affects movement probability
    }

    /// Add energy (from food)
    pub fn add_energy(&mut self, amount: f64) {
        self.energy += amount;
    }

    /// Check if creature is dead
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Take damage (reduces health)
    pub fn take_damage(&mut self, amount: f64) {
        self.health -= amount;
    }

    /// Check if creature can reproduce
    pub fn can_reproduce(&self, threshold: f64) -> bool {
        self.energy >= threshold
    }

    /// Reproduce with another creature, consuming energy
    /// Child spawns at the average position of the two parents
    pub fn reproduce(&mut self, other: &mut Creature, mutation_rate: f64) -> Option<Creature> {
        if !self.can_reproduce(60.0) || !other.can_reproduce(60.0) {
            return None;
        }

        // Reproduce costs energy - reduced from 50.0 to 20.0 for easier reproduction
        let cost = 20.0;
        self.energy -= cost;
        other.energy -= cost;

        let mut child_genome = self.genome.crossover(&other.genome);
        child_genome.mutate(mutation_rate);

        // Child inherits genome_id from one of the parents (for lineage tracking)
        let child_genome_id = self.genome_id;

        // Spawn at average position
        let child_x = (self.x + other.x) / 2;
        let child_y = (self.y + other.y) / 2;

        Some(Creature::new(
            child_genome,
            child_genome_id,
            child_x,
            child_y,
        ))
    }

    /// Calculate fitness score
    pub fn fitness(&self) -> f64 {
        self.genome.fitness_score()
    }

    /// Get combat power (for resource competition)
    pub fn combat_power(&self) -> f64 {
        self.genome.strength + self.genome.size * 0.5
    }

    /// Get vision radius (affected by size)
    pub fn vision_radius(&self) -> f64 {
        5.0 + self.genome.size * 10.0 // Base 5 + up to 10 more
    }

    /// Calculate movement success probability based on speed and energy
    /// Zero energy = 1/10th of normal probability
    /// 1 or more energy = normal probability
    pub fn movement_probability(&self) -> f64 {
        let base_probability = 0.3 + (self.genome.speed * 0.7); // 30% to 100% chance
        if self.energy <= 0.0 {
            base_probability * 0.1 // 1/10th probability when out of energy
        } else {
            base_probability
        }
    }

    /// Attempt to move in a direction
    /// Returns new position if successful, None if failed or out of bounds
    pub fn try_move<R: Rng>(
        &self,
        direction: Direction,
        world_width: usize,
        world_height: usize,
        rng: &mut R,
    ) -> Option<(usize, usize)> {
        // Check if movement succeeds
        if rng.gen::<f64>() > self.movement_probability() {
            return None;
        }

        let (dx, dy) = match direction {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        };

        let new_x = self.x as i32 + dx;
        let new_y = self.y as i32 + dy;

        // Check bounds
        if new_x >= 0 && new_x < world_width as i32 && new_y >= 0 && new_y < world_height as i32 {
            Some((new_x as usize, new_y as usize))
        } else {
            None
        }
    }

    /// Find best direction to move towards a target
    pub fn direction_to(&self, target_x: usize, target_y: usize) -> Direction {
        let dx = target_x as i32 - self.x as i32;
        let dy = target_y as i32 - self.y as i32;

        match (dx.signum(), dy.signum()) {
            (0, -1) => Direction::North,
            (0, 1) => Direction::South,
            (1, 0) => Direction::East,
            (-1, 0) => Direction::West,
            (1, -1) => Direction::NorthEast,
            (-1, -1) => Direction::NorthWest,
            (1, 1) => Direction::SouthEast,
            (-1, 1) => Direction::SouthWest,
            _ => Direction::North, // Already at target or edge case
        }
    }

    /// Calculate distance to a point
    pub fn distance_to(&self, x: usize, y: usize) -> f64 {
        let dx = x as f64 - self.x as f64;
        let dy = y as f64 - self.y as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_creation() {
        let genome = Genome::default();
        let genome_id = Uuid::new_v4();
        let creature = Creature::new(genome, genome_id, 10, 10);
        assert_eq!(creature.energy, 100.0);
        assert_eq!(creature.health, 100.0);
        assert_eq!(creature.x, 10);
        assert_eq!(creature.y, 10);
    }

    #[test]
    fn test_energy_consumption() {
        let genome = Genome::default();
        let genome_id = Uuid::new_v4();
        let mut creature = Creature::new(genome, genome_id, 10, 10);
        let initial_energy = creature.energy;

        creature.consume_energy();

        // Energy consumption is disabled - energy should stay the same
        assert_eq!(creature.energy, initial_energy);
    }

    #[test]
    fn test_death() {
        let genome = Genome::default();
        let genome_id = Uuid::new_v4();
        let mut creature = Creature::new(genome, genome_id, 10, 10);
        creature.health = 0.0;

        assert!(creature.is_dead());
    }

    #[test]
    fn test_reproduction() {
        let genome = Genome::default();
        let genome_id = Uuid::new_v4();
        let mut parent1 = Creature::new(genome.clone(), genome_id, 10, 10);
        let mut parent2 = Creature::new(genome, genome_id, 15, 15);

        parent1.energy = 150.0;
        parent2.energy = 150.0;

        let child = parent1.reproduce(&mut parent2, 0.1);

        assert!(child.is_some());
        assert!(parent1.energy < 150.0);
        assert!(parent2.energy < 150.0);
    }

    #[test]
    fn test_reproduction_requires_energy() {
        let genome = Genome::default();
        let genome_id = Uuid::new_v4();
        let mut parent1 = Creature::new(genome.clone(), genome_id, 10, 10);
        let mut parent2 = Creature::new(genome, genome_id, 15, 15);

        parent1.energy = 50.0; // Below the 60.0 threshold
        parent2.energy = 50.0;

        let child = parent1.reproduce(&mut parent2, 0.1);

        assert!(child.is_none());
    }

    #[test]
    fn test_movement_probability() {
        let genome = Genome {
            speed: 1.0,
            ..Default::default()
        };
        let genome_id = Uuid::new_v4();
        let creature = Creature::new(genome, genome_id, 10, 10);

        // With normal energy (100.0), should be full probability
        assert_eq!(creature.movement_probability(), 1.0);

        let genome2 = Genome {
            speed: 0.0,
            ..Default::default()
        };
        let mut creature2 = Creature::new(genome2, Uuid::new_v4(), 10, 10);

        // With normal energy, should be base 0.3
        assert_eq!(creature2.movement_probability(), 0.3);

        // With zero energy, should be 1/10th
        creature2.energy = 0.0;
        assert_eq!(creature2.movement_probability(), 0.03);
    }

    #[test]
    fn test_direction_to() {
        let genome = Genome::default();
        let genome_id = Uuid::new_v4();
        let creature = Creature::new(genome, genome_id, 10, 10);

        assert_eq!(creature.direction_to(10, 5), Direction::North);
        assert_eq!(creature.direction_to(10, 15), Direction::South);
        assert_eq!(creature.direction_to(15, 10), Direction::East);
        assert_eq!(creature.direction_to(5, 10), Direction::West);
        assert_eq!(creature.direction_to(15, 5), Direction::NorthEast);
    }
}

use shared::Genome;

#[derive(Debug, Clone)]
pub struct Creature {
    pub genome: Genome,
    pub energy: f64,
    pub age: u32,
}

impl Creature {
    /// Create a new creature with the given genome
    pub fn new(genome: Genome) -> Self {
        Self {
            genome,
            energy: 50.0, // Starting energy
            age: 0,
        }
    }

    /// Consume energy based on genome
    pub fn consume_energy(&mut self) {
        self.energy -= self.genome.energy_cost();
        self.age += 1;
    }

    /// Add energy (from food)
    pub fn add_energy(&mut self, amount: f64) {
        self.energy += amount;
    }

    /// Check if creature is dead
    pub fn is_dead(&self) -> bool {
        self.energy <= 0.0
    }

    /// Check if creature can reproduce
    pub fn can_reproduce(&self, threshold: f64) -> bool {
        self.energy >= threshold
    }

    /// Reproduce with another creature, consuming energy
    pub fn reproduce(&mut self, other: &mut Creature, mutation_rate: f64) -> Option<Creature> {
        if !self.can_reproduce(100.0) || !other.can_reproduce(100.0) {
            return None;
        }

        // Reproduce costs energy
        let cost = 50.0;
        self.energy -= cost;
        other.energy -= cost;

        let mut child_genome = self.genome.crossover(&other.genome);
        child_genome.mutate(mutation_rate);

        Some(Creature::new(child_genome))
    }

    /// Calculate fitness score
    pub fn fitness(&self) -> f64 {
        self.genome.fitness_score()
    }

    /// Get combat power (for resource competition)
    pub fn combat_power(&self) -> f64 {
        self.genome.strength + self.genome.size * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_creation() {
        let genome = Genome::default();
        let creature = Creature::new(genome);
        assert_eq!(creature.energy, 50.0);
        assert_eq!(creature.age, 0);
    }

    #[test]
    fn test_energy_consumption() {
        let genome = Genome::default();
        let mut creature = Creature::new(genome);
        let initial_energy = creature.energy;

        creature.consume_energy();

        assert!(creature.energy < initial_energy);
        assert_eq!(creature.age, 1);
    }

    #[test]
    fn test_death() {
        let genome = Genome::default();
        let mut creature = Creature::new(genome);
        creature.energy = 0.0;

        assert!(creature.is_dead());
    }

    #[test]
    fn test_reproduction() {
        let genome = Genome::default();
        let mut parent1 = Creature::new(genome.clone());
        let mut parent2 = Creature::new(genome);

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
        let mut parent1 = Creature::new(genome.clone());
        let mut parent2 = Creature::new(genome);

        parent1.energy = 50.0;
        parent2.energy = 50.0;

        let child = parent1.reproduce(&mut parent2, 0.1);

        assert!(child.is_none());
    }
}

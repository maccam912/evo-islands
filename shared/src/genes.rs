use rand::Rng;
use serde::{Deserialize, Serialize};

/// A genome represents the genetic makeup of a creature.
/// Each gene has a value between 0.0 and 1.0.
///
/// Design Philosophy:
/// - Higher strength/speed/size provides advantages but increases energy costs
/// - Efficiency reduces energy costs
/// - Reproduction affects breeding rate but costs energy
/// - This creates a balancing act and arms race dynamics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    /// Physical strength - increases combat power but costs energy
    pub strength: f64,

    /// Movement speed - helps escape/chase but costs energy
    pub speed: f64,

    /// Body size - provides more health but increases energy needs
    pub size: f64,

    /// Energy efficiency - reduces overall energy consumption
    pub efficiency: f64,

    /// Reproduction rate - affects breeding frequency
    pub reproduction: f64,
}

impl Genome {
    /// Create a new random genome
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            strength: rng.gen(),
            speed: rng.gen(),
            size: rng.gen(),
            efficiency: rng.gen(),
            reproduction: rng.gen(),
        }
    }

    /// Create a genome with specific values (clamped to 0.0-1.0)
    pub fn new(strength: f64, speed: f64, size: f64, efficiency: f64, reproduction: f64) -> Self {
        Self {
            strength: strength.clamp(0.0, 1.0),
            speed: speed.clamp(0.0, 1.0),
            size: size.clamp(0.0, 1.0),
            efficiency: efficiency.clamp(0.0, 1.0),
            reproduction: reproduction.clamp(0.0, 1.0),
        }
    }

    /// Mutate this genome by adding random noise
    pub fn mutate(&mut self, mutation_rate: f64) {
        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < mutation_rate {
            self.strength = (self.strength + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f64>() < mutation_rate {
            self.speed = (self.speed + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f64>() < mutation_rate {
            self.size = (self.size + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f64>() < mutation_rate {
            self.efficiency = (self.efficiency + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f64>() < mutation_rate {
            self.reproduction = (self.reproduction + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
    }

    /// Cross two genomes to create offspring
    pub fn crossover(&self, other: &Genome) -> Genome {
        let mut rng = rand::thread_rng();
        Genome {
            strength: if rng.gen() {
                self.strength
            } else {
                other.strength
            },
            speed: if rng.gen() { self.speed } else { other.speed },
            size: if rng.gen() { self.size } else { other.size },
            efficiency: if rng.gen() {
                self.efficiency
            } else {
                other.efficiency
            },
            reproduction: if rng.gen() {
                self.reproduction
            } else {
                other.reproduction
            },
        }
    }

    /// Calculate the energy cost per tick for this genome
    /// Higher strength, speed, and size increase cost
    /// Higher efficiency decreases cost
    pub fn energy_cost(&self) -> f64 {
        let base_cost = 1.0;
        let trait_cost =
            self.strength * 2.0 + self.speed * 1.5 + self.size * 1.8 + self.reproduction * 0.5;
        let efficiency_multiplier = 2.0 - self.efficiency; // 1.0 to 2.0

        base_cost + trait_cost * efficiency_multiplier
    }

    /// Calculate fitness score (higher is better)
    /// This is a complex balance of all traits
    pub fn fitness_score(&self) -> f64 {
        // Combat effectiveness
        let combat = self.strength + self.size * 0.5;
        // Survival ability
        let survival = self.speed + self.efficiency;
        // Reproduction capability
        let breeding = self.reproduction;

        // Balanced fitness that doesn't favor any single strategy
        (combat * survival * breeding).powf(1.0 / 3.0)
    }
}

impl Default for Genome {
    fn default() -> Self {
        Self {
            strength: 0.5,
            speed: 0.5,
            size: 0.5,
            efficiency: 0.5,
            reproduction: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_creation() {
        let genome = Genome::new(0.8, 0.6, 0.7, 0.5, 0.4);
        assert_eq!(genome.strength, 0.8);
        assert_eq!(genome.speed, 0.6);
    }

    #[test]
    fn test_genome_clamping() {
        let genome = Genome::new(1.5, -0.5, 0.5, 0.5, 0.5);
        assert_eq!(genome.strength, 1.0);
        assert_eq!(genome.speed, 0.0);
    }

    #[test]
    fn test_energy_cost() {
        let high_cost = Genome::new(1.0, 1.0, 1.0, 0.0, 1.0);
        let low_cost = Genome::new(0.0, 0.0, 0.0, 1.0, 0.0);

        assert!(high_cost.energy_cost() > low_cost.energy_cost());
    }

    #[test]
    fn test_crossover() {
        let parent1 = Genome::new(1.0, 0.0, 1.0, 0.0, 1.0);
        let parent2 = Genome::new(0.0, 1.0, 0.0, 1.0, 0.0);

        let child = parent1.crossover(&parent2);

        // Child should have values from one parent or the other
        assert!(child.strength == 1.0 || child.strength == 0.0);
        assert!(child.speed == 0.0 || child.speed == 1.0);
    }

    #[test]
    fn test_mutation_maintains_bounds() {
        let mut genome = Genome::new(0.5, 0.5, 0.5, 0.5, 0.5);

        for _ in 0..100 {
            genome.mutate(1.0); // Always mutate

            assert!(genome.strength >= 0.0 && genome.strength <= 1.0);
            assert!(genome.speed >= 0.0 && genome.speed <= 1.0);
            assert!(genome.size >= 0.0 && genome.size <= 1.0);
            assert!(genome.efficiency >= 0.0 && genome.efficiency <= 1.0);
            assert!(genome.reproduction >= 0.0 && genome.reproduction <= 1.0);
        }
    }
}

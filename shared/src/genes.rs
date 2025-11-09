use rand::Rng;
use serde::{Deserialize, Serialize};

/// Total trait budget - the sum of all traits should equal this value
/// With 5 traits, a budget of 2.5 means an average of 0.5 per trait
/// This forces trade-offs: high values in some traits means low values in others
const TRAIT_BUDGET: f64 = 2.5;

/// A genome represents the genetic makeup of a creature.
/// Each gene has a value between 0.0 and 1.0.
///
/// Design Philosophy:
/// - Higher strength/speed/size provides advantages but increases energy costs
/// - Efficiency reduces energy costs
/// - Reproduction affects breeding rate but costs energy
/// - **Trait Budget**: All traits sum to a fixed budget, forcing strategic trade-offs
/// - This creates diverse strategies and prevents all traits from maxing out
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
    /// Normalize traits to fit within the TRAIT_BUDGET while keeping traits in [0, 1]
    /// This enforces trade-offs: high values in some traits means low values in others
    fn normalize(&mut self) {
        // Clamp all values to [0, 1] first
        self.strength = self.strength.clamp(0.0, 1.0);
        self.speed = self.speed.clamp(0.0, 1.0);
        self.size = self.size.clamp(0.0, 1.0);
        self.efficiency = self.efficiency.clamp(0.0, 1.0);
        self.reproduction = self.reproduction.clamp(0.0, 1.0);

        let sum = self.strength + self.speed + self.size + self.efficiency + self.reproduction;

        // Scale proportionally to meet TRAIT_BUDGET, then clamp again
        if sum > 0.0 {
            let scale = TRAIT_BUDGET / sum;
            self.strength = (self.strength * scale).min(1.0);
            self.speed = (self.speed * scale).min(1.0);
            self.size = (self.size * scale).min(1.0);
            self.efficiency = (self.efficiency * scale).min(1.0);
            self.reproduction = (self.reproduction * scale).min(1.0);
        } else {
            // If all traits are 0, distribute evenly
            let even_split = TRAIT_BUDGET / 5.0;
            self.strength = even_split;
            self.speed = even_split;
            self.size = even_split;
            self.efficiency = even_split;
            self.reproduction = even_split;
        }
    }

    /// Create a new random genome with balanced traits
    /// Traits will sum to TRAIT_BUDGET, forcing strategic trade-offs
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut genome = Self {
            strength: rng.gen(),
            speed: rng.gen(),
            size: rng.gen(),
            efficiency: rng.gen(),
            reproduction: rng.gen(),
        };
        genome.normalize();
        genome
    }

    /// Create a genome with specific values (normalized to fit TRAIT_BUDGET)
    pub fn new(strength: f64, speed: f64, size: f64, efficiency: f64, reproduction: f64) -> Self {
        let mut genome = Self {
            strength: strength.clamp(0.0, 1.0),
            speed: speed.clamp(0.0, 1.0),
            size: size.clamp(0.0, 1.0),
            efficiency: efficiency.clamp(0.0, 1.0),
            reproduction: reproduction.clamp(0.0, 1.0),
        };
        genome.normalize();
        genome
    }

    /// Mutate this genome by adding random noise while maintaining TRAIT_BUDGET
    /// Mutations shift trait values, creating trade-offs between different traits
    pub fn mutate(&mut self, mutation_rate: f64) {
        let mut rng = rand::thread_rng();
        let mut mutated = false;

        if rng.gen::<f64>() < mutation_rate {
            self.strength = (self.strength + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
            mutated = true;
        }
        if rng.gen::<f64>() < mutation_rate {
            self.speed = (self.speed + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
            mutated = true;
        }
        if rng.gen::<f64>() < mutation_rate {
            self.size = (self.size + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
            mutated = true;
        }
        if rng.gen::<f64>() < mutation_rate {
            self.efficiency = (self.efficiency + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
            mutated = true;
        }
        if rng.gen::<f64>() < mutation_rate {
            self.reproduction = (self.reproduction + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
            mutated = true;
        }

        // Re-normalize to maintain trait budget after mutations
        if mutated {
            self.normalize();
        }
    }

    /// Cross two genomes to create offspring with normalized traits
    pub fn crossover(&self, other: &Genome) -> Genome {
        let mut rng = rand::thread_rng();
        let mut child = Genome {
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
        };
        child.normalize();
        child
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
        // After normalization, traits should sum close to TRAIT_BUDGET (or less if clamped)
        let sum = genome.strength + genome.speed + genome.size + genome.efficiency + genome.reproduction;
        assert!(sum <= TRAIT_BUDGET + 0.001, "Traits sum should not exceed TRAIT_BUDGET");
        assert!(sum >= TRAIT_BUDGET * 0.8, "Traits sum should be reasonably close to TRAIT_BUDGET");
        // Traits should maintain their relative proportions
        assert!(genome.strength > genome.speed);
        assert!(genome.size > genome.efficiency);
    }

    #[test]
    fn test_genome_clamping() {
        let genome = Genome::new(1.5, -0.5, 0.5, 0.5, 0.5);
        // Values should be clamped and normalized
        assert!(genome.strength >= 0.0 && genome.strength <= 1.0);
        assert!(genome.speed >= 0.0 && genome.speed <= 1.0);
        let sum = genome.strength + genome.speed + genome.size + genome.efficiency + genome.reproduction;
        assert!(sum <= TRAIT_BUDGET + 0.001, "Traits sum should not exceed TRAIT_BUDGET");
        assert!(sum >= TRAIT_BUDGET * 0.8, "Traits sum should be reasonably close to TRAIT_BUDGET");
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

        // Child traits should be within valid bounds
        assert!(child.strength >= 0.0 && child.strength <= 1.0);
        assert!(child.speed >= 0.0 && child.speed <= 1.0);
        assert!(child.size >= 0.0 && child.size <= 1.0);
        assert!(child.efficiency >= 0.0 && child.efficiency <= 1.0);
        assert!(child.reproduction >= 0.0 && child.reproduction <= 1.0);
        // Child should maintain trait budget constraint (with lower threshold for extreme parents)
        let sum = child.strength + child.speed + child.size + child.efficiency + child.reproduction;
        assert!(sum <= TRAIT_BUDGET + 0.001, "Child traits sum should not exceed TRAIT_BUDGET");
        assert!(sum > 0.0, "Child should have some trait values");
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

            // Verify trait budget is maintained after mutation
            let sum = genome.strength + genome.speed + genome.size + genome.efficiency + genome.reproduction;
            assert!(sum <= TRAIT_BUDGET + 0.001, "Trait sum should not exceed TRAIT_BUDGET after mutation");
            assert!(sum >= TRAIT_BUDGET * 0.5, "Trait sum should be reasonably close to TRAIT_BUDGET after mutation");
        }
    }
}

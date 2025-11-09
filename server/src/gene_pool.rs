use shared::{Genome, GenomeWithFitness, GenomeWithId, GlobalStats, SurvivalResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Entry in the population-tracked gene pool
#[derive(Debug, Clone)]
struct GenomeEntry {
    genome: Genome,
    population: u32, // Virtual population size
}

/// Manages the global gene pool with population tracking
#[derive(Clone)]
pub struct GenePool {
    inner: Arc<RwLock<GenePoolInner>>,
}

struct GenePoolInner {
    /// All genomes tracked by ID with their populations
    genomes: HashMap<Uuid, GenomeEntry>,

    /// Active clients
    active_clients: std::collections::HashSet<Uuid>,

    /// Statistics
    total_work_units: u64,
    total_simulations: u64,

    /// Server start time
    start_time: std::time::Instant,
}

impl GenePool {
    pub fn new() -> Self {
        let mut genomes = HashMap::new();

        // Start with 10 random genomes with initial populations
        for _ in 0..10 {
            let genome_id = Uuid::new_v4();
            let genome = Genome::random();
            genomes.insert(
                genome_id,
                GenomeEntry {
                    genome,
                    population: 100,
                },
            );
        }

        Self {
            inner: Arc::new(RwLock::new(GenePoolInner {
                genomes,
                active_clients: std::collections::HashSet::new(),
                total_work_units: 0,
                total_simulations: 0,
                start_time: std::time::Instant::now(),
            })),
        }
    }

    /// Get seed genomes for a new spatial simulation
    /// Server injects mutation into seeds to prevent client-side tampering.
    pub async fn get_seed_genomes_spatial(&self) -> Vec<GenomeWithId> {
        let inner = self.inner.read().await;

        // Separate living and extinct genomes
        let mut living: Vec<_> = inner
            .genomes
            .values()
            .filter(|e| e.population > 0)
            .collect();
        let extinct: Vec<_> = inner
            .genomes
            .values()
            .filter(|e| e.population == 0)
            .collect();

        // Sort living by population
        living.sort_by(|a, b| b.population.cmp(&a.population));

        println!(
            "Gene pool: {} living (pop range: {}-{}), {} extinct, {} total",
            living.len(),
            living.last().map(|e| e.population).unwrap_or(0),
            living.first().map(|e| e.population).unwrap_or(0),
            extinct.len(),
            inner.genomes.len()
        );

        // Build base selection: 5 living + 5 extinct
        let mut base: Vec<Genome> = living.iter().take(5).map(|e| e.genome.clone()).collect();
        use rand::seq::SliceRandom;
        let mut extinct_pick: Vec<Genome> = {
            let mut rng = rand::thread_rng();
            extinct
                .choose_multiple(&mut rng, 5)
                .map(|e| e.genome.clone())
                .collect()
        };
        base.append(&mut extinct_pick);
        drop(inner); // release read lock

        while base.len() < 10 {
            base.push(Genome::random());
        }
        if base.len() > 10 {
            base.truncate(10);
        }

        // Mutate seeds server-side
        const SERVER_MUT_RATE: f64 = 0.05;
        let mut out: Vec<GenomeWithId> = Vec::with_capacity(10);
        let mut new_entries: Vec<(Uuid, Genome)> = Vec::with_capacity(10);
        for g in base.into_iter() {
            let mut mg = g.clone();
            mg.mutate(SERVER_MUT_RATE);
            let id = Uuid::new_v4();
            new_entries.push((id, mg.clone()));
            out.push(GenomeWithId {
                genome_id: id,
                genome: mg,
            });
        }

        // Insert mutated genomes into pool with population 0
        let mut inner_write = self.inner.write().await;
        for (genome_id, genome) in new_entries {
            inner_write.genomes.insert(
                genome_id,
                GenomeEntry {
                    genome,
                    population: 0,
                },
            );
        }

        out
    }

    /// Get seed genomes (legacy method for backwards compatibility)
    #[allow(dead_code)]
    pub async fn get_seed_genomes(&self, count: usize) -> Vec<Genome> {
        let inner = self.inner.read().await;

        let mut seeds = Vec::new();

        // Take living genomes sorted by population
        let mut living: Vec<_> = inner
            .genomes
            .values()
            .filter(|e| e.population > 0)
            .collect();

        living.sort_by(|a, b| b.population.cmp(&a.population));

        for entry in living.iter().take(count) {
            seeds.push(entry.genome.clone());
        }

        // Fill with random if needed
        while seeds.len() < count {
            seeds.push(Genome::random());
        }

        seeds
    }

    /// Submit survival results from a spatial simulation
    pub async fn submit_survival_results(
        &self,
        client_id: Uuid,
        survival_results: Vec<SurvivalResult>,
        steps_completed: u32,
        best_genomes: Vec<GenomeWithFitness>,
    ) {
        let mut inner = self.inner.write().await;

        inner.total_work_units += 1;
        inner.total_simulations += steps_completed as u64;
        inner.active_clients.insert(client_id);

        // Update populations based on survival
        let mut population_changes = Vec::new();
        for result in survival_results {
            if let Some(entry) = inner.genomes.get_mut(&result.genome_id) {
                let old_pop = entry.population;
                if result.survived > 0 {
                    // Survivors: boost population
                    entry.population = entry.population.saturating_add(result.survived * 10);
                    population_changes.push((result.genome_id, old_pop, entry.population, result.survived));
                } else {
                    // Extinct: reduce population
                    entry.population = entry.population.saturating_sub(20);
                    population_changes.push((result.genome_id, old_pop, entry.population, 0));
                }
            } else {
                // Unknown genome - this shouldn't happen but handle gracefully
                eprintln!(
                    "Warning: Received results for unknown genome {}",
                    result.genome_id
                );
            }
        }

        if !population_changes.is_empty() {
            println!("Population updates:");
            for (id, old_pop, new_pop, survived) in population_changes {
                println!("  {} {} → {} (survived: {})",
                    &id.to_string()[..8], old_pop, new_pop, survived);
            }
        }

        // Ingest reported best genomes as new entries
        // Give them a meaningful starting population so they can be selected and tested
        if !best_genomes.is_empty() {
            let min_fitness = best_genomes.iter().map(|g| g.fitness).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);
            let max_fitness = best_genomes.iter().map(|g| g.fitness).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);

            println!(
                "Ingested {} evolved genomes with population=50 (fitness range: {:.3}-{:.3})",
                best_genomes.len(),
                min_fitness,
                max_fitness
            );
        }

        for gwf in best_genomes {
            let id = Uuid::new_v4();
            inner.genomes.insert(
                id,
                GenomeEntry {
                    genome: gwf.genome,
                    population: 50, // Start with population boost to enable selection
                },
            );
        }

        // Limit max population to prevent overflow
        for entry in inner.genomes.values_mut() {
            if entry.population > 10000 {
                entry.population = 10000;
            }
        }
    }

    /// Submit results (legacy method for backwards compatibility)
    pub async fn submit_results(
        &self,
        client_id: Uuid,
        _best_genomes: Vec<GenomeWithFitness>,
        generations_completed: u32,
    ) {
        let mut inner = self.inner.write().await;

        inner.total_work_units += 1;
        inner.total_simulations += generations_completed as u64;
        inner.active_clients.insert(client_id);

        // Legacy method doesn't update populations
    }

    /// Get global statistics
    pub async fn get_stats(&self) -> GlobalStats {
        let inner = self.inner.read().await;

        // Get top genomes by population
        let mut entries: Vec<_> = inner.genomes.values().collect();
        entries.sort_by(|a, b| b.population.cmp(&a.population));

        let best_genomes: Vec<GenomeWithFitness> = entries
            .iter()
            .take(10)
            .map(|e| GenomeWithFitness {
                genome: e.genome.clone(),
                fitness: e.genome.fitness_score(), // For display purposes
            })
            .collect();

        GlobalStats {
            active_clients: inner.active_clients.len(),
            total_work_units: inner.total_work_units,
            total_generations: inner.total_simulations,
            best_genomes,
            gene_pool_size: inner.genomes.len(),
            uptime_seconds: inner.start_time.elapsed().as_secs(),
            unique_genomes: inner.genomes.len(),
        }
    }

    /// Register a client as active
    pub async fn register_client(&self, client_id: Uuid) {
        let mut inner = self.inner.write().await;
        inner.active_clients.insert(client_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gene_pool_creation() {
        let pool = GenePool::new();
        let stats = pool.get_stats().await;

        assert!(stats.gene_pool_size > 0);
        assert!(!stats.best_genomes.is_empty());
    }

    #[tokio::test]
    async fn test_get_seed_genomes_spatial() {
        let pool = GenePool::new();
        let seeds = pool.get_seed_genomes_spatial().await;

        assert_eq!(seeds.len(), 10);
    }

    #[tokio::test]
    async fn test_submit_survival_results() {
        let pool = GenePool::new();
        let client_id = Uuid::new_v4();

        // Get a genome ID from the pool
        let seeds = pool.get_seed_genomes_spatial().await;
        let genome_id = seeds[0].genome_id;

        let results = vec![SurvivalResult {
            genome_id,
            survived: 5,
            total_spawned: 10,
            avg_lifespan: 100.0,
            total_food_eaten: 500,
        }];

        pool.submit_survival_results(client_id, results, 3000, Vec::new())
            .await;

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_work_units, 1);
    }

    #[tokio::test]
    async fn test_population_updates() {
        let pool = GenePool::new();
        let client_id = Uuid::new_v4();

        let seeds = pool.get_seed_genomes_spatial().await;
        let genome_id = seeds[0].genome_id;

        // Initial population
        let _initial_stats = pool.get_stats().await;
        let initial_pop = {
            let inner = pool.inner.read().await;
            inner.genomes.get(&genome_id).unwrap().population
        };

        // Simulate survival
        let results = vec![SurvivalResult {
            genome_id,
            survived: 3,
            total_spawned: 5,
            avg_lifespan: 200.0,
            total_food_eaten: 300,
        }];

        pool.submit_survival_results(client_id, results, 3000, Vec::new())
            .await;

        // Check population increased
        let new_pop = {
            let inner = pool.inner.read().await;
            inner.genomes.get(&genome_id).unwrap().population
        };

        assert!(new_pop > initial_pop);
    }

    #[tokio::test]
    async fn test_extinction() {
        let pool = GenePool::new();
        let client_id = Uuid::new_v4();

        let seeds = pool.get_seed_genomes_spatial().await;
        let genome_id = seeds[0].genome_id;

        // Simulate extinction (no survivors)
        let results = vec![SurvivalResult {
            genome_id,
            survived: 0,
            total_spawned: 1,
            avg_lifespan: 10.0,
            total_food_eaten: 0,
        }];

        pool.submit_survival_results(client_id, results, 3000, Vec::new())
            .await;

        // Check population decreased
        let new_pop = {
            let inner = pool.inner.read().await;
            inner.genomes.get(&genome_id).unwrap().population
        };

        // Population should have decreased by 20
        assert!(new_pop < 100);
    }
}

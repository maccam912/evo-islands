use rand::seq::SliceRandom;
use shared::{Genome, GenomeWithFitness, GlobalStats};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Manages the global gene pool and tracks statistics
#[derive(Clone)]
pub struct GenePool {
    inner: Arc<RwLock<GenePoolInner>>,
}

struct GenePoolInner {
    /// Best genomes discovered so far (sorted by fitness)
    best_genomes: Vec<GenomeWithFitness>,

    /// Historical genomes (older genomes that were once good)
    historical_genomes: Vec<Genome>,

    /// Active clients
    active_clients: std::collections::HashSet<Uuid>,

    /// Statistics
    total_work_units: u64,
    total_generations: u64,

    /// Server start time
    start_time: std::time::Instant,
}

impl GenePool {
    pub fn new() -> Self {
        let mut best_genomes = Vec::new();

        // Start with some random genomes
        for _ in 0..10 {
            let genome = Genome::random();
            best_genomes.push(GenomeWithFitness {
                fitness: genome.fitness_score(),
                genome,
            });
        }

        best_genomes.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        Self {
            inner: Arc::new(RwLock::new(GenePoolInner {
                best_genomes,
                historical_genomes: Vec::new(),
                active_clients: std::collections::HashSet::new(),
                total_work_units: 0,
                total_generations: 0,
                start_time: std::time::Instant::now(),
            })),
        }
    }

    /// Get seed genomes for a new work assignment
    /// Returns best genomes plus some random historical ones
    pub async fn get_seed_genomes(&self, count: usize) -> Vec<Genome> {
        let inner = self.inner.read().await;

        let mut seeds = Vec::new();

        // Take best genomes (70% of seeds)
        let best_count = (count as f64 * 0.7) as usize;
        for genome_with_fitness in inner.best_genomes.iter().take(best_count) {
            seeds.push(genome_with_fitness.genome.clone());
        }

        // Take random historical genomes (30% of seeds)
        let historical_count = count - seeds.len();
        if !inner.historical_genomes.is_empty() {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            let chosen: Vec<_> = inner
                .historical_genomes
                .choose_multiple(&mut rng, historical_count)
                .cloned()
                .collect();
            seeds.extend(chosen);
        }

        // Fill remaining with random if needed
        while seeds.len() < count {
            seeds.push(Genome::random());
        }

        seeds
    }

    /// Submit results from a completed work unit
    pub async fn submit_results(
        &self,
        client_id: Uuid,
        best_genomes: Vec<GenomeWithFitness>,
        generations_completed: u32,
    ) {
        let mut inner = self.inner.write().await;

        inner.total_work_units += 1;
        inner.total_generations += generations_completed as u64;
        inner.active_clients.insert(client_id);

        // Add new genomes to the pool
        for new_genome in best_genomes {
            // Check if this is better than any existing genome
            let should_add = inner.best_genomes.is_empty()
                || new_genome.fitness > inner.best_genomes.last().unwrap().fitness
                || inner.best_genomes.len() < 100;

            if should_add {
                inner.best_genomes.push(new_genome.clone());

                // Keep best 100 genomes
                inner
                    .best_genomes
                    .sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
                if inner.best_genomes.len() > 100 {
                    // Move evicted genomes to historical pool
                    if let Some(evicted) = inner.best_genomes.pop() {
                        inner.historical_genomes.push(evicted.genome);
                    }
                }
            }
        }

        // Keep historical pool manageable
        if inner.historical_genomes.len() > 1000 {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            inner.historical_genomes.shuffle(&mut rng);
            inner.historical_genomes.truncate(1000);
        }
    }

    /// Get global statistics
    pub async fn get_stats(&self) -> GlobalStats {
        let inner = self.inner.read().await;

        GlobalStats {
            active_clients: inner.active_clients.len(),
            total_work_units: inner.total_work_units,
            total_generations: inner.total_generations,
            best_genomes: inner.best_genomes.iter().take(10).cloned().collect(),
            gene_pool_size: inner.best_genomes.len() + inner.historical_genomes.len(),
            uptime_seconds: inner.start_time.elapsed().as_secs(),
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
    async fn test_get_seed_genomes() {
        let pool = GenePool::new();
        let seeds = pool.get_seed_genomes(5).await;

        assert_eq!(seeds.len(), 5);
    }

    #[tokio::test]
    async fn test_submit_results() {
        let pool = GenePool::new();
        let client_id = Uuid::new_v4();

        let results = vec![GenomeWithFitness {
            genome: Genome::new(0.9, 0.8, 0.7, 0.6, 0.5),
            fitness: 0.8,
        }];

        pool.submit_results(client_id, results, 100).await;

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_work_units, 1);
        assert_eq!(stats.total_generations, 100);
    }
}

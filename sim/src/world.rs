use rand::Rng;

/// Types of tiles in the world
#[derive(Debug, Clone, PartialEq)]
pub enum Tile {
    Empty,
    /// Renewable resource: regrows food over time (current_food, max_food, ticks_until_regrowth)
    Plant {
        current_food: u32,
        max_food: u32,
        regrowth_timer: u32,
    },
    /// Consumable resource: doesn't regrow
    Food { amount: u32 },
}

/// 2D grid world for spatial simulation
#[derive(Debug, Clone)]
pub struct World {
    pub width: usize,
    pub height: usize,
    grid: Vec<Vec<Tile>>,
}

impl World {
    /// Create a new world with specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![Tile::Empty; width]; height];
        World {
            width,
            height,
            grid,
        }
    }

    /// Initialize world with plants and food scattered randomly
    /// plant_density: percentage of tiles that are plants (e.g., 0.05 = 5%)
    /// food_density: percentage of tiles that are consumable food (e.g., 0.02 = 2%)
    pub fn initialize_resources<R: Rng>(
        &mut self,
        rng: &mut R,
        plant_density: f64,
        food_density: f64,
    ) {
        let total_tiles = self.width * self.height;
        let num_plants = (total_tiles as f64 * plant_density) as usize;
        let num_food = (total_tiles as f64 * food_density) as usize;

        // Place plants
        for _ in 0..num_plants {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if matches!(self.grid[y][x], Tile::Empty) {
                self.grid[y][x] = Tile::Plant {
                    current_food: 10,
                    max_food: 10,
                    regrowth_timer: 0,
                };
            }
        }

        // Place consumable food
        for _ in 0..num_food {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if matches!(self.grid[y][x], Tile::Empty) {
                self.grid[y][x] = Tile::Food {
                    amount: rng.gen_range(5..=15),
                };
            }
        }
    }

    /// Get tile at position (returns None if out of bounds)
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            Some(&self.grid[y][x])
        } else {
            None
        }
    }

    /// Get mutable tile at position
    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < self.width && y < self.height {
            Some(&mut self.grid[y][x])
        } else {
            None
        }
    }

    /// Check if position is valid
    pub fn is_valid_position(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    /// Try to eat food from a tile
    /// Returns the amount of food available (0 if none)
    pub fn get_available_food(&self, x: usize, y: usize) -> u32 {
        match self.get_tile(x, y) {
            Some(Tile::Plant { current_food, .. }) => *current_food,
            Some(Tile::Food { amount }) => *amount,
            _ => 0,
        }
    }

    /// Consume food from a tile
    /// Returns the amount actually consumed
    pub fn consume_food(&mut self, x: usize, y: usize, amount_requested: u32) -> u32 {
        if let Some(tile) = self.get_tile_mut(x, y) {
            match tile {
                Tile::Plant {
                    current_food,
                    regrowth_timer,
                    ..
                } => {
                    let consumed = (*current_food).min(amount_requested);
                    *current_food -= consumed;
                    // Start regrowth timer when depleted
                    if *current_food == 0 {
                        *regrowth_timer = 10; // Takes 10 ticks to regrow 1 food
                    }
                    consumed
                }
                Tile::Food { amount } => {
                    let consumed = (*amount).min(amount_requested);
                    *amount -= consumed;
                    // Remove tile if depleted
                    if *amount == 0 {
                        *tile = Tile::Empty;
                    }
                    consumed
                }
                _ => 0,
            }
        } else {
            0
        }
    }

    /// Update all plants - regrow food over time
    pub fn tick_plants(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Tile::Plant {
                    current_food,
                    max_food,
                    regrowth_timer,
                } = &mut self.grid[y][x]
                {
                    if *current_food < *max_food {
                        if *regrowth_timer > 0 {
                            // Count down and regrow exactly when hitting zero
                            *regrowth_timer -= 1;
                            if *regrowth_timer == 0 {
                                *current_food += 1;
                                if *current_food < *max_food {
                                    *regrowth_timer = 10;
                                }
                            }
                        } else {
                            // Immediate regrowth path when timer already zero
                            *current_food += 1;
                            if *current_food < *max_food {
                                *regrowth_timer = 10;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Find all food positions within a radius of a point
    pub fn find_food_in_radius(&self, center_x: usize, center_y: usize, radius: f64) -> Vec<(usize, usize, u32)> {
        let mut food_positions = Vec::new();
        let radius_squared = radius * radius;

        let min_x = center_x.saturating_sub(radius.ceil() as usize);
        let max_x = (center_x + radius.ceil() as usize).min(self.width - 1);
        let min_y = center_y.saturating_sub(radius.ceil() as usize);
        let max_y = (center_y + radius.ceil() as usize).min(self.height - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Check if within radius
                let dx = x as f64 - center_x as f64;
                let dy = y as f64 - center_y as f64;
                let dist_squared = dx * dx + dy * dy;

                if dist_squared <= radius_squared {
                    let food = self.get_available_food(x, y);
                    if food > 0 {
                        food_positions.push((x, y, food));
                    }
                }
            }
        }

        food_positions
    }

    /// Get total food available in the world
    pub fn total_food(&self) -> u32 {
        let mut total = 0;
        for row in &self.grid {
            for tile in row {
                match tile {
                    Tile::Plant { current_food, .. } => total += current_food,
                    Tile::Food { amount } => total += amount,
                    _ => {}
                }
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_world_creation() {
        let world = World::new(100, 100);
        assert_eq!(world.width, 100);
        assert_eq!(world.height, 100);
    }

    #[test]
    fn test_initialize_resources() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut world = World::new(100, 100);
        world.initialize_resources(&mut rng, 0.05, 0.02);

        let total = world.total_food();
        assert!(total > 0, "World should have food after initialization");
    }

    #[test]
    fn test_consume_food() {
        let mut world = World::new(10, 10);
        world.grid[5][5] = Tile::Food { amount: 10 };

        let consumed = world.consume_food(5, 5, 5);
        assert_eq!(consumed, 5);
        assert_eq!(world.get_available_food(5, 5), 5);

        let consumed = world.consume_food(5, 5, 10);
        assert_eq!(consumed, 5);
        assert_eq!(world.get_available_food(5, 5), 0);
        assert!(matches!(world.get_tile(5, 5), Some(Tile::Empty)));
    }

    #[test]
    fn test_plant_regrowth() {
        let mut world = World::new(10, 10);
        world.grid[5][5] = Tile::Plant {
            current_food: 10,
            max_food: 10,
            regrowth_timer: 0,
        };

        // Consume all food
        world.consume_food(5, 5, 10);
        assert_eq!(world.get_available_food(5, 5), 0);

        // Tick 10 times to trigger regrowth
        for _ in 0..10 {
            world.tick_plants();
        }

        assert_eq!(world.get_available_food(5, 5), 1);

        // Continue ticking to regrow more
        for _ in 0..10 {
            world.tick_plants();
        }

        assert_eq!(world.get_available_food(5, 5), 2);
    }

    #[test]
    fn test_find_food_in_radius() {
        let mut world = World::new(20, 20);
        world.grid[10][10] = Tile::Food { amount: 5 };
        world.grid[12][10] = Tile::Food { amount: 3 };
        world.grid[15][15] = Tile::Food { amount: 7 };

        let food = world.find_food_in_radius(10, 10, 3.0);
        assert_eq!(food.len(), 2); // Should find (10,10) and (12,10)

        let food = world.find_food_in_radius(10, 10, 10.0);
        assert_eq!(food.len(), 3); // Should find all three
    }
}

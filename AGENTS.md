# EvoIslands Codebase Guide for AI Agents

This document provides a comprehensive overview of the EvoIslands project structure, components, and implementation details for future AI agent development and maintenance.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Project Structure](#project-structure)
3. [Architecture](#architecture)
4. [Source Files and Modules](#source-files-and-modules)
5. [API Endpoints](#api-endpoints)
6. [Data Structures](#data-structures)
7. [Component Interactions](#component-interactions)
8. [Configuration Files](#configuration-files)
9. [Dependencies](#dependencies)
10. [Key Implementation Details](#key-implementation-details)

---

## Project Overview

**EvoIslands** is a distributed evolution simulation platform inspired by Folding@Home. It enables distributed computing of genetic algorithm simulations across multiple clients with a central server managing the global gene pool.

**Key Characteristics:**
- Distributed computation using Folding@Home-style work distribution
- Island-based evolution model where each client simulates evolution independently
- Global gene pool management on the server
- Real-time web dashboard for monitoring
- Kubernetes deployment ready
- Automatic client version checking
- Protocol versioning for compatibility

**Core Concept:** Creatures compete on isolated "islands" with five evolving traits (strength, speed, size, efficiency, reproduction). Results feed back to the server's global gene pool, and new work assignments include the best genomes to seed new simulations.

---

## Project Structure

```
evo-islands/
├── shared/              # Shared types and protocol definitions
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs       # Module exports
│       ├── genes.rs     # Genome structure and genetic operations
│       └── protocol.rs  # Network communication types
├── sim/                 # Evolution simulation engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs       # Main simulation orchestrator
│       ├── creature.rs  # Creature lifecycle management
│       └── island.rs    # Island simulation logic
├── server/              # Central server implementation
│   ├── Cargo.toml
│   ├── Dockerfile       # Multi-stage build for server binary
│   ├── src/
│   │   ├── main.rs      # Server entry point and initialization
│   │   ├── server.rs    # HTTP API endpoint handlers
│   │   ├── gene_pool.rs # Global gene pool state management
│   │   └── web.rs       # Web UI and health check handlers
│   └── static/
│       └── index.html   # Web dashboard UI
├── client/              # Client implementation
│   ├── Cargo.toml
│   ├── Dockerfile       # Multi-stage build for client binary
│   └── src/
│       ├── main.rs      # Client entry point
│       ├── client.rs    # Work request/submission and simulation processing
│       └── tui.rs       # TUI placeholder (currently unused, logging instead)
├── k8s/                 # Kubernetes deployment manifests
│   ├── server-deployment.yaml
│   └── client-deployment.yaml
├── .github/
│   └── workflows/
│       └── ci.yml       # GitHub Actions CI/CD pipeline
├── Cargo.toml           # Workspace configuration
├── Cargo.lock           # Locked dependencies
├── rustfmt.toml         # Code formatting rules
├── .gitignore           # Git ignore patterns
└── README.md            # User-facing documentation
```

---

## Architecture

### High-Level Design

```
┌─────────────┐
│   Clients   │  (Multiple instances)
│ (Simulation)│  - Request work from server
│  Execution  │  - Run island simulations
└──────┬──────┘  - Submit best genomes
       │
       │ HTTP API (REST)
       │ Request work / Submit results
       │
       ▼
┌──────────────────┐
│   EvoIslands     │
│   Server         │
├──────────────────┤
│ Gene Pool Mgmt   │  - Maintains best genomes (top 100)
│ Work Distribution│  - Stores historical genomes (1000)
│ Statistics       │  - Tracks active clients
│ Web Dashboard    │  - Serves real-time stats
└──────────────────┘
       │
       │ Served to Browser
       │
       ▼
┌──────────────────┐
│  Web Dashboard   │  - Real-time stats
│ (index.html)     │  - Top creatures display
│                  │  - SVG creature visualization
└──────────────────┘
```

### Component Architecture

**Four independent crates with clear dependencies:**

```
shared (no dependencies on others)
  ↓ (used by all others)
  ├─ server
  │   ├─ uses: sim
  │   └─ provides: HTTP API + Web UI
  ├─ client
  │   ├─ uses: sim
  │   └─ provides: Simulation execution
  └─ sim
      └─ provides: Island simulation logic
```

---

## Source Files and Modules

### 1. SHARED CRATE (`/home/user/evo-islands/shared/`)

**Purpose:** Protocol definitions, data structures, and genetic algorithms shared across all components.

#### `/home/user/evo-islands/shared/src/lib.rs`
- Module exports
- `PROTOCOL_VERSION` constant (currently: 1) - must match between client and server
- Re-exports all public types from submodules

#### `/home/user/evo-islands/shared/src/genes.rs`
- **`Genome` struct:** Represents creature genetics with 5 genes
  - `strength: f64` (0.0-1.0): Combat power, high energy cost
  - `speed: f64` (0.0-1.0): Movement speed, high energy cost
  - `size: f64` (0.0-1.0): Body size/health, high energy cost
  - `efficiency: f64` (0.0-1.0): Energy efficiency multiplier
  - `reproduction: f64` (0.0-1.0): Breeding rate frequency
- **Key methods:**
  - `random()`: Create random genome
  - `new(s, sp, sz, e, r)`: Create with values (auto-clamped to 0-1)
  - `mutate(rate)`: Add random noise to genes (+-0.1 per gene if triggered)
  - `crossover(other)`: Offspring inherits each gene randomly from parent
  - `energy_cost()`: Calculate per-tick energy drain (1.0 base + trait costs * efficiency)
  - `fitness_score()`: Balanced fitness metric = (combat * survival * breeding)^(1/3)
- **Design Philosophy:** No single trait dominates; all strategies have tradeoffs

#### `/home/user/evo-islands/shared/src/protocol.rs`
- **`WorkRequest`**: Client → Server
  - `client_id: Uuid`: Persistent client identifier
  - `protocol_version: u32`: Version check (must match `PROTOCOL_VERSION`)
  - `client_version: String`: Version string from build
- **`WorkAssignment`**: Server → Client
  - `work_id: Uuid`: Unique work unit identifier
  - `seed_genomes: Vec<Genome>`: Initial population (best 70% + historical 30%)
  - `generations: u32`: How many generations to simulate (100)
  - `population_size: usize`: Island population (50)
  - `mutation_rate: f64`: Mutation probability per gene (0.05 = 5%)
- **`WorkResult`**: Client → Server
  - `work_id: Uuid`: Reference to assigned work
  - `client_id: Uuid`: Client identifier
  - `best_genomes: Vec<GenomeWithFitness>`: Top genomes discovered
  - `generations_completed: u32`: Actual generations executed
  - `stats: SimulationStats`: Aggregated metrics
- **`GenomeWithFitness`**: Genome paired with fitness score
- **`SimulationStats`**: Work result statistics
  - `avg_fitness: f64`: Average across all generations
  - `best_fitness: f64`: Peak fitness achieved
  - `final_population: usize`: Living creatures at end
  - `total_creatures: usize`: Sum of all creatures that lived
- **`GlobalStats`**: Server → Browser (via `/api/stats`)
  - `active_clients: usize`: Currently connected clients
  - `total_work_units: u64`: Completed work units
  - `total_generations: u64`: Cumulative generations
  - `best_genomes: Vec<GenomeWithFitness>`: Top 10 genomes
  - `gene_pool_size: usize`: Total stored genomes
  - `uptime_seconds: u64`: Server runtime
- **`ServerError`**: Error responses
  - `VersionMismatch`: Protocol version incompatibility (causes client exit)
  - `ServerOverloaded`: Temporary unavailability
  - `InvalidRequest(msg)`: Bad request format
  - `InternalError(msg)`: Server-side error

---

### 2. SIM CRATE (`/home/user/evo-islands/sim/`)

**Purpose:** Evolution simulation engine - runs island-based genetic algorithms.

#### `/home/user/evo-islands/sim/src/lib.rs`
- **`run_simulation(genomes, generations, pop_size, mutation_rate) → (best_genomes, stats)`**
  - Main entry point for client processing
  - Creates `Island` with configuration
  - Runs specified number of generations (calling `island.tick()` each)
  - Tracks fitness statistics across generations
  - Returns top 10 best genomes and aggregated statistics
  - Configuration details:
    - `food_per_tick = population_size * 0.8` (slight resource scarcity)
    - `reproduction_threshold = 100.0` (energy needed to breed)
    - `max_age = 500` (maximum generation lifespan)

#### `/home/user/evo-islands/sim/src/creature.rs`
- **`Creature` struct:**
  - `genome: Genome`: Genetic information
  - `energy: f64`: Current energy level (dies at 0)
  - `age: u32`: Ticks lived
- **Lifecycle methods:**
  - `new(genome)`: Create with 50.0 initial energy, age 0
  - `consume_energy()`: Deduct genome's energy cost, increment age
  - `add_energy(amount)`: Gain food energy
  - `is_dead()`: Check if energy <= 0
  - `can_reproduce(threshold)`: Check if energy >= threshold (100.0)
  - `reproduce(other, mutation_rate)`: Create offspring if both have energy
    - Both parents lose 50.0 energy
    - Offspring created via crossover + mutation
    - Returns `Option<Creature>` (None if insufficient energy)
  - `fitness()`: Delegate to `genome.fitness_score()`
  - `combat_power()`: Calculate combat (strength + size*0.5) for food competition

#### `/home/user/evo-islands/sim/src/island.rs`
- **`IslandConfig` struct:**
  - `population_size: usize`: Target population (50)
  - `mutation_rate: f64`: Per-gene mutation rate (0.05)
  - `food_per_tick: f64`: Total resource per generation
  - `reproduction_threshold: f64`: Energy needed to breed (100.0)
  - `max_age: u32`: Maximum creature lifespan (500)
- **`Island` struct:**
  - `config: IslandConfig`: Configuration
  - `creatures: Vec<Creature>`: Current population
  - `generation: u32`: Current generation counter
- **Key methods:**
  - `new(config, seed_genomes)`: Initialize with seed population
  - `tick()`: Execute one generation cycle:
    1. All creatures consume energy
    2. Distribute food based on combat power (competition)
    3. Remove dead and aged-out creatures
    4. Reproduction phase (random pairing)
    5. Maintain minimum population (add random if < pop_size/4)
  - `distribute_food()`: Allocate resources
    - Creatures with higher combat power receive more food
    - Equal distribution if no combat stats
  - `reproduce()`: Breeding phase
    - Shuffle creatures for random pairings
    - Check energy requirements
    - Create offspring from adjacent pairs
  - `average_fitness()`: Mean fitness of population
  - `get_best_genomes(n)`: Return top N genomes sorted by fitness

**Evolution Mechanics:**
- Resources are scarce (80% of population size per tick)
- Stronger creatures win food but pay high energy costs
- Reproduction requires 100 energy from both parents, costs 50 each
- Maximum age prevents immortality
- Mutation is per-gene with ±0.1 change
- Selection happens naturally through survival and reproduction

---

### 3. SERVER CRATE (`/home/user/evo-islands/server/`)

**Purpose:** Central coordination server managing work distribution, gene pool, and web interface.

#### `/home/user/evo-islands/server/src/main.rs`
- Entry point
- Initializes tracing/logging with env-filter
- Calls `server::run()` on tokio runtime
- Default log level: "server=debug,tower_http=debug"

#### `/home/user/evo-islands/server/src/server.rs`
- **`AppState` struct:**
  - `gene_pool: GenePool`: Shared mutable state (Arc<RwLock<>>)
- **Axum router setup:**
  - Port: 8080
  - CORS: Permissive (allows all origins)
  - Routes:
    - `POST /api/work/request`: Handle work requests
    - `POST /api/work/submit`: Handle result submission
    - `GET /api/stats`: Get global statistics
    - `GET /health`, `GET /healthz`: Kubernetes health probes
    - `GET /`: Serve index.html web UI
- **Handler: `handle_work_request()`**
  - Validates protocol version (returns 400 if mismatch)
  - Registers client as active
  - Gets 10 seed genomes from gene pool
  - Creates `WorkAssignment`:
    - 100 generations
    - 50 population size
    - 5% mutation rate
  - Returns JSON assignment
- **Handler: `handle_work_submit()`**
  - Accepts `WorkResult` from client
  - Submits to gene pool (updates best genomes, stats)
  - Returns 200 OK
- **Handler: `handle_stats()`**
  - Returns `GlobalStats` from gene pool
  - Called by dashboard and health checks
- **Error handling:**
  - `ApiError` enum with version mismatch support
  - Converts to HTTP responses with JSON error body

#### `/home/user/evo-islands/server/src/gene_pool.rs`
- **`GenePool` struct (thread-safe):**
  - Wraps `Arc<RwLock<GenePoolInner>>`
  - Inner contains:
    - `best_genomes: Vec<GenomeWithFitness>`: Top 100 genomes (sorted by fitness desc)
    - `historical_genomes: Vec<Genome>`: Previous best genomes (max 1000)
    - `active_clients: HashSet<Uuid>`: Currently connected clients
    - `total_work_units: u64`: Completed work units count
    - `total_generations: u64`: Cumulative generations
    - `start_time: Instant`: Server startup time
- **Key methods:**
  - `new()`: Initialize with 10 random genomes
  - `get_seed_genomes(count)`: Prepare seeds for work assignment
    - 70% best genomes (top 7 from best_genomes)
    - 30% historical genomes (random from historical_genomes)
    - Fill remainder with random if needed
  - `submit_results(client_id, best_genomes, generations)`
    - Increments work unit and generation counters
    - Adds new genomes to best_genomes if better than worst or pool < 100
    - Keeps top 100 only (evicted genomes move to historical)
    - Maintains historical pool at max 1000 (shuffles and truncates if needed)
  - `get_stats()`: Return `GlobalStats` snapshot
  - `register_client(client_id)`: Mark client active
- **Gene pool strategy:**
  - Elitism: Always keep top 100
  - Historical diversity: Maintain 1000 older genomes
  - Seeding bias: New work gets 70% best to guide evolution
  - Diversity: 30% historical to maintain genetic variation

#### `/home/user/evo-islands/server/src/web.rs`
- **`index()` handler:** Serves `static/index.html` with `include_str!` macro (embedded in binary)
- **`health()` handler:** Returns `{"status": "healthy"}` for Kubernetes probes

#### `/home/user/evo-islands/server/static/index.html`
- **Framework:** Vanilla HTML/CSS/JavaScript (no build step needed)
- **Styling:**
  - Purple gradient background (667eea to 764ba2)
  - Glass-morphism cards (frosted glass effect)
  - Responsive grid layout
  - SVG creature visualization
- **Real-time updates:**
  - Fetches `/api/stats` every 2 seconds
  - Displays:
    - Active client count
    - Work units completed (formatted: K/M notation)
    - Total generations
    - Gene pool size
    - Server uptime (formatted as h:mm or mm)
  - Top 6 evolved creatures with:
    - Rank badges (medal emojis for top 3, #4+ for others)
    - SVG visualization (body size = size gene, color = hue(strength*360), lightness = efficiency)
    - 5 gene bar charts (strength, speed, size, efficiency, reproduction)
    - Fitness score display
- **Creature visualization:**
  - SVG body with ellipse (size affects shape)
  - Head and eyes
  - Leg indicators (length/thickness from speed)
  - Arm indicators (thickness from strength)
  - Color hue based on strength, lightness based on efficiency

---

### 4. CLIENT CRATE (`/home/user/evo-islands/client/`)

**Purpose:** Workers that request work, run simulations, and submit results.

#### `/home/user/evo-islands/client/src/main.rs`
- Entry point
- Initializes logging (default: "client=info")
- Gets server URL from `SERVER_URL` env var (default: `https://evo-islands.rackspace.koski.co`)
- Calls `client::run(server_url)`

#### `/home/user/evo-islands/client/src/client.rs`
- **`Client` struct:**
  - `client_id: Uuid`: Unique persistent identifier (generated per session, not persisted)
  - `server_url: String`: Server endpoint
  - `http_client: reqwest::Client`: HTTP client (30-second timeout)
- **Methods:**
  - `new(server_url)`: Create client
  - `request_work()`: Make `WorkRequest` to `/api/work/request`
    - Returns `WorkAssignment` on success
    - Handles version mismatch error (logs error, returns error to trigger exit)
    - Handles overload errors (logs warning, returns error for retry)
    - Returns parsed `WorkAssignment`
  - `submit_results(result)`: POST `WorkResult` to `/api/work/submit`
    - Returns `Ok(())` on 200 status
    - Logs error and returns error on failure
  - `process_work(assignment)`: Run simulation synchronously
    - Calls `sim::run_simulation()` with assignment parameters
    - Returns `WorkResult` with work_id, client_id, best_genomes, stats
- **`run(server_url)` main loop:**
  - Creates client
  - Infinite loop:
    1. Request work (10-second retry delay on failure)
    2. Check for version mismatch (exit if detected)
    3. Process work locally (run simulation)
    4. Submit results (5-second retry delay on failure)
    5. Log "Work completed successfully"

#### `/home/user/evo-islands/client/src/tui.rs`
- **Status:** Placeholder file
- **Note:** Currently unused; client uses logging instead of TUI
- **Future enhancement:** Could implement ratatui-based status display

---

## API Endpoints

All endpoints use JSON for request/response bodies. Base URL: server address (e.g., `http://localhost:8080`).

### Work Distribution

#### `POST /api/work/request`
**Request body:**
```json
{
  "client_id": "uuid",
  "protocol_version": 1,
  "client_version": "0.1.0"
}
```
**Success response (200):**
```json
{
  "work_id": "uuid",
  "seed_genomes": [
    {
      "strength": 0.5,
      "speed": 0.6,
      "size": 0.7,
      "efficiency": 0.4,
      "reproduction": 0.5
    }
  ],
  "generations": 100,
  "population_size": 50,
  "mutation_rate": 0.05
}
```
**Error response (400) - Version mismatch:**
```json
{
  "VersionMismatch": {
    "server_version": 1,
    "client_version": 0
  }
}
```

#### `POST /api/work/submit`
**Request body:**
```json
{
  "work_id": "uuid",
  "client_id": "uuid",
  "best_genomes": [
    {
      "genome": { "strength": 0.8, "speed": 0.7, ... },
      "fitness": 0.75
    }
  ],
  "generations_completed": 100,
  "stats": {
    "avg_fitness": 0.65,
    "best_fitness": 0.85,
    "final_population": 42,
    "total_creatures": 520
  }
}
```
**Success response (200):** Empty body

### Monitoring

#### `GET /api/stats`
**Response (200):**
```json
{
  "active_clients": 5,
  "total_work_units": 127,
  "total_generations": 12700,
  "best_genomes": [
    {
      "genome": { "strength": 0.9, "speed": 0.8, ... },
      "fitness": 0.92
    }
  ],
  "gene_pool_size": 150,
  "uptime_seconds": 86400
}
```

### Web UI

#### `GET /`
**Response (200):** HTML dashboard (index.html)

### Health Checks

#### `GET /health`
#### `GET /healthz`
**Response (200):**
```json
{
  "status": "healthy"
}
```

---

## Data Structures

### Core Genetics

**Genome** (in shared/src/genes.rs)
- 5 traits: strength, speed, size, efficiency, reproduction
- All values: 0.0 to 1.0 (clamped)
- Methods: random(), new(), mutate(), crossover(), energy_cost(), fitness_score()
- Energy cost formula: `(1.0 + (str*2 + spd*1.5 + sz*1.8 + rep*0.5) * (2 - eff))`
- Fitness formula: `((combat * survival * breeding) ^ (1/3))`
  - combat = strength + size*0.5
  - survival = speed + efficiency
  - breeding = reproduction

### Creature State

**Creature** (in sim/src/creature.rs)
- genome: Genome
- energy: f64 (dies at 0)
- age: u32

### Island State

**Island** (in sim/src/island.rs)
- creatures: Vec<Creature>
- generation: u32
- config: IslandConfig

**IslandConfig**
- population_size: usize (50)
- mutation_rate: f64 (0.05)
- food_per_tick: f64 (pop_size * 0.8)
- reproduction_threshold: f64 (100.0)
- max_age: u32 (500)

### Network Protocol

**WorkRequest** - Client asks for work
**WorkAssignment** - Server gives work
**WorkResult** - Client submits results
**GenomeWithFitness** - Genome + score
**SimulationStats** - Metrics from simulation
**GlobalStats** - Server state snapshot
**ServerError** - Error responses

---

## Component Interactions

### Full Work Cycle

```
1. CLIENT
   ├─ Creates Client with server URL
   └─ Enters infinite loop
     
2. REQUEST WORK
   ├─ POST /api/work/request with client_id, protocol_version
   ├─ Server validates protocol_version (PROTOCOL_VERSION = 1)
   ├─ Server calls gene_pool.register_client(client_id)
   ├─ Server calls gene_pool.get_seed_genomes(10)
   │  └─ Returns: 7 best genomes + 3 random historical
   ├─ Server creates WorkAssignment (100 gen, 50 pop, 0.05 mut)
   └─ Client receives WorkAssignment
     
3. PROCESS WORK (Local)
   ├─ Client calls sim::run_simulation(seeds, 100, 50, 0.05)
   ├─ Sim creates Island with seed genomes
   ├─ Sim runs 100 generations:
   │  ├─ Each generation calls island.tick()
   │  │  ├─ Creatures consume energy
   │  │  ├─ Food distributed by combat power
   │  │  ├─ Dead/old creatures removed
   │  │  ├─ Breeding phase (random pairs)
   │  │  └─ Population minimum maintained
   │  └─ Track fitness stats
   ├─ Sim returns top 10 genomes + stats
   └─ Client receives results locally
     
4. SUBMIT RESULTS
   ├─ Client POSTs WorkResult to /api/work/submit
   ├─ Server calls gene_pool.submit_results()
   │  ├─ Increments total_work_units
   ├─ Increments total_generations += 100
   │  ├─ Adds client to active_clients
   │  ├─ Merges best_genomes with global pool
   │  │  ├─ Keeps top 100
   │  │  └─ Moves evicted to historical
   │  └─ Trims historical to 1000 max
   └─ Server returns 200 OK
     
5. DASHBOARD UPDATE
   ├─ Browser polls /api/stats every 2 seconds
   ├─ Server returns GlobalStats snapshot
   ├─ Dashboard renders:
   │  ├─ Active clients count
   │  ├─ Work units (formatted)
   │  ├─ Total generations (formatted)
   │  ├─ Gene pool size
   │  ├─ Uptime
   │  └─ Top 6 creatures with SVG visualization
   └─ Cycle repeats every 2 seconds

6. VERSION UPDATE
   ├─ Server updates PROTOCOL_VERSION constant
   ├─ New clients request work with new version
   ├─ Old clients get VersionMismatch error
   ├─ Old clients log error and exit(1)
   ├─ Kubernetes restarts pod with new image
   └─ New client joins with new version
```

### Data Flow

**Client → Server:**
- WorkRequest: JSON POST with client metadata
- WorkResult: JSON POST with simulation results

**Server → Client:**
- WorkAssignment: JSON response with work parameters
- GlobalStats: JSON response for monitoring (also served to dashboard)
- ServerError: JSON error response

**Server → Browser:**
- HTML: index.html on GET /
- JSON: /api/stats every 2 seconds

---

## Configuration Files

### `/home/user/evo-islands/Cargo.toml`
- **Workspace config** for all crates
- Defines shared dependencies with workspace.dependencies
- Members: shared, sim, server, client
- Edition: 2021
- License: MIT

### `/home/user/evo-islands/Cargo.lock`
- Lock file for reproducible builds
- Not committed (gitignored)
- Regenerated on build

### `/home/user/evo-islands/rustfmt.toml`
- Edition: 2021
- max_width: 100 characters

### `/home/user/evo-islands/.gitignore`
- Rust: /target/, *.rs.bk, *.pdb, Cargo.lock
- IDE: .vscode/, .idea/, *.swp, *.swo, *~
- OS: .DS_Store, Thumbs.db
- Logs: *.log

### `/home/user/evo-islands/k8s/server-deployment.yaml`
- **Deployment:** Single replica of server
- **Image:** ghcr.io/maccam912/evo-islands-server:latest
- **Port:** 8080
- **Resources:**
  - Request: 256Mi memory, 250m CPU
  - Limit: 512Mi memory, 500m CPU
- **Health probes:**
  - Liveness: GET /api/stats every 30s (initial 10s delay)
  - Readiness: GET /api/stats every 10s (initial 5s delay)
- **Service:** ClusterIP on port 8080
- **Ingress:** nginx with TLS via cert-manager
  - Host: evo-islands.rackspace.koski.co
  - Secret: evo-islands-tls

### `/home/user/evo-islands/k8s/client-deployment.yaml`
- **Deployment:** 3 client replicas (scalable)
- **Image:** ghcr.io/maccam912/evo-islands-client:latest
- **Environment:**
  - SERVER_URL: https://evo-islands.rackspace.koski.co
  - RUST_LOG: client=info
- **Resources:** Same as server (256/512 Mi mem, 250/500m CPU)
- **Restart:** Always

### `/home/user/evo-islands/.github/workflows/ci.yml`
- **Triggers:** Push to main or claude/** branches, PR to main
- **Test job:**
  - Checkout, install Rust
  - Cache cargo registry/index/target
  - Run `cargo test --all --verbose`
  - Check formatting: `cargo fmt --all -- --check`
  - Run clippy: `cargo clippy --all --all-targets -- -D warnings`
- **Build jobs (only on push to main/claude/**):**
  - build-server: Docker build & push server image to ghcr.io
  - build-client: Docker build & push client image to ghcr.io
  - Tags: latest (for main), sha hash
  - Requires passing test job
  - Uses GitHub packages (GHCR) registry

### `/home/user/evo-islands/server/Dockerfile`
- **Build stage:** rust:1.85-slim
  - Copy all workspace files
  - Run `cargo build --release -p server`
- **Runtime stage:** debian:bookworm-slim
  - Install ca-certificates, libssl3
  - Copy binary from builder
  - EXPOSE 8080
  - CMD: /app/server

### `/home/user/evo-islands/client/Dockerfile`
- **Build stage:** rust:1.85-slim
  - Copy all workspace files
  - Run `cargo build --release -p client`
- **Runtime stage:** debian:bookworm-slim
  - Install ca-certificates, libssl3
  - Copy binary from builder
  - ENV SERVER_URL=https://evo-islands.rackspace.koski.co
  - CMD: /app/client

---

## Dependencies

### Core Dependencies (Workspace)

**Runtime:**
- **tokio** (1.35): Async runtime with full features
- **axum** (0.7): HTTP framework for server
- **tower** (0.4): Service middleware ecosystem
- **tower-http** (0.5): HTTP middleware (CORS, file serving)
- **reqwest** (0.11): HTTP client for client with JSON support
- **serde** (1.0): Serialization framework
- **serde_json** (1.0): JSON support
- **uuid** (1.6): Unique identifiers (v4, serde support)
- **rand** (0.8): Random number generation
- **anyhow** (1.0): Error handling
- **thiserror** (1.0): Custom error types
- **tracing** (0.1): Structured logging
- **tracing-subscriber** (0.3): Logging implementation with env-filter

**UI Client Only:**
- **ratatui** (0.26): Terminal UI framework (currently unused)
- **crossterm** (0.27): Terminal manipulation (currently unused)

**Testing:**
- **proptest** (1.4): Property-based testing

### Version Constraints

- Rust Edition: 2021
- Rust Version: 1.85+ (specified in Dockerfiles)
- MSRV (Minimum Supported Rust Version): Not explicitly stated, likely ~1.75+

---

## Key Implementation Details

### Version Control and Compatibility

**Protocol Version (shared/src/lib.rs):**
- `PROTOCOL_VERSION = 1`
- Clients send their protocol_version in WorkRequest
- Server validates at `/api/work/request` endpoint
- Mismatch → `ServerError::VersionMismatch` (400 Bad Request)
- Client detects version mismatch and exits (triggers Kubernetes restart)
- **Strategy:** Version checking ensures distributed clients stay compatible

### Energy Economy and Evolution Dynamics

**Energy Cost Per Tick:**
- Base: 1.0
- Trait contribution: strength*2.0 + speed*1.5 + size*1.8 + reproduction*0.5
- Efficiency multiplier: 2.0 - efficiency (ranges 1.0-2.0)
- Formula: `base + (traits * multiplier)`
- **Effect:** High-power creatures need constant feeding

**Competition:**
- Food per tick: population_size * 0.8 (20% resource scarcity)
- Distribution: Proportional to combat_power (strength + size*0.5)
- **Effect:** Stronger creatures eat better but cost more

**Reproduction:**
- Threshold: 100 energy minimum for each parent
- Cost: 50 energy per parent, 1 new creature
- **Effect:** Breeding is expensive, limits population growth

**Fitness Scoring:**
- Combat: strength + size*0.5
- Survival: speed + efficiency
- Breeding: reproduction
- Score: `(combat * survival * breeding) ^ (1/3)`
- **Effect:** Balanced fitness, no single dominant strategy

### Concurrency Model

**Server Gene Pool:**
- Uses `Arc<RwLock<>>` for thread-safe shared state
- Multiple tokio tasks can read concurrently (RwLock read guard)
- Single writer at a time (submit_results, register_client)
- **Performance:** High read concurrency for stats queries

**Client:**
- Single-threaded event loop with tokio
- Blocking simulation (can upgrade to spawn_blocking if needed)
- HTTP requests are async

### State Management

**Gene Pool Persistence:**
- **In-memory only** (lost on server restart)
- No database layer
- Good for simulation, not for production durability
- **Could add:** Periodic snapshots to disk/database

**Client ID:**
- Generated per session (not persisted)
- **Could improve:** Save to file for persistent identification

### Testing

**All crates have unit tests:**
- Genome: creation, clamping, mutation, crossover, energy
- Creature: lifecycle, reproduction, combat
- Island: tick mechanics, fitness, best genomes
- Protocol: serialization, deserialization
- Server: work request handler, version validation
- Client: client creation, work processing

**Test command:** `cargo test --all --verbose`

### Error Handling

**Client-side:**
- anyhow::Result for most operations
- Logs errors, continues on network failures
- Exits on version mismatch
- Retries with backoff on server overload

**Server-side:**
- ApiError enum with Into<Response>
- Returns appropriate HTTP status codes
- JSON error bodies with details

### Logging and Observability

**Client:**
- Default level: "client=info"
- Configurable via RUST_LOG env var
- Logs: work requests, results, errors

**Server:**
- Default level: "server=debug,tower_http=debug"
- Configurable via RUST_LOG env var
- Logs: work requests, client registration, stats queries

**Dashboard:**
- Real-time stats via /api/stats polling
- Visual creature representations
- Uptime and generation tracking

### Deployment Model

**Kubernetes:**
- Single server, multiple clients
- Horizontal scaling: Add client replicas with kubectl scale
- Vertical scaling: Increase client CPU for faster simulation
- Health probes: Liveness and readiness on /api/stats

**Docker:**
- Multi-stage builds for minimal images
- Base: debian:bookworm-slim runtime
- Build: rust:1.85-slim
- Server: Exposes 8080
- Client: Configured via SERVER_URL env var

**CI/CD (GitHub Actions):**
- Tests on every push and PR
- Builds Docker images on push to main or claude/** branches
- Pushes to GitHub Container Registry (GHCR)
- Automatic rollout via Kubernetes image pull

---

## Quick Reference for Developers

### Adding a New Gene Trait

1. Add field to `Genome` struct (shared/src/genes.rs)
2. Update `new()`, `mutate()`, `crossover()` methods
3. Adjust `energy_cost()` formula
4. Adjust `fitness_score()` if needed
5. Add tests
6. Update `index.html` visualization if visual

### Adding an API Endpoint

1. Create handler function in server/src/server.rs
2. Add route to Axum router in `run()`
3. Define request/response types (in shared if needed)
4. Add tests

### Tuning Evolution

- Edit `run_simulation()` in sim/src/lib.rs:
  - `food_per_tick`: Resource scarcity
  - `reproduction_threshold`: Breeding difficulty
  - `max_age`: Lifespan

- Edit `WorkAssignment` creation in server/src/server.rs:
  - `generations`: Work unit size (100 is good)
  - `population_size`: Island population (50 is good)
  - `mutation_rate`: Evolution rate (0.05 = 5% is good)

### Testing Locally

```bash
# Run all tests
cargo test --all

# Run server
cd server && cargo run --release

# Run client (in another terminal)
cd client && cargo run --release

# Visit dashboard
open http://localhost:8080
```

---

## Future Enhancement Opportunities

1. **Database persistence** for gene pool and statistics
2. **TUI implementation** in client for status display
3. **Advanced visualization** of evolution over time
4. **Genetic algorithm tuning** UI controls
5. **Multi-version support** for gradual rollouts
6. **Client authentication** and credits system
7. **Advanced selection strategies** (tournament, etc.)
8. **Metrics export** (Prometheus, OpenTelemetry)
9. **Work queue prioritization** based on diversity needs
10. **Island isolation modes** for different evolution experiments

---

## Summary

EvoIslands is a well-architected distributed computing platform with clear separation of concerns:

- **shared**: Protocol and genetics (no dependencies)
- **sim**: Pure simulation engine (depends on shared)
- **server**: Coordination and monitoring (depends on shared, sim)
- **client**: Worker executable (depends on shared, sim)

The system uses a simple HTTP REST API, in-memory gene pool management, and automatic version checking for compatibility. The web dashboard provides real-time visualization of evolving creatures and system metrics. The design favors simplicity and horizontal scalability through Kubernetes.

Key architectural decisions:
- Shared types for protocol compatibility
- Token-based work units for distributed execution
- Elitism (keep best) + diversity (historical pool)
- Energy-based metabolism for realistic evolution
- Real-time web dashboard with SVG visualization
- Automatic client version management

This is production-ready code suitable for educational purposes and distributed computing experiments.

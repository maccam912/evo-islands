# 🧬 EvoIslands - Distributed Evolution Simulation

A distributed evolution experiment inspired by Folding@Home, where clients simulate evolution on isolated "islands" and report the best genomes back to a central server.

## 🌟 Features

- **Distributed Computing**: Folding@Home-style work distribution
- **Island Evolution**: Clients simulate evolution in isolated environments
- **Gene Pool Management**: Server tracks and distributes the best genomes
- **Balanced Evolution**: Creatures must balance power, speed, size, efficiency, and reproduction
- **Web Dashboard**: Real-time visualization of evolution progress and best creatures
- **Version Control**: Automatic client updates when server version changes
- **Kubernetes Ready**: Full deployment manifests included

## 🏗️ Architecture

### Components

1. **Server** (`server/`)
   - Distributes work to clients
   - Manages global gene pool
   - Tracks best genomes and statistics
   - Serves web UI for monitoring

2. **Client** (`client/`)
   - Requests work from server
   - Runs evolution simulations
   - Reports best genomes back
   - Auto-restarts on version mismatch

3. **Simulation Engine** (`sim/`)
   - Island-based evolution
   - Creature lifecycle management
   - Resource competition
   - Mutation and crossover

4. **Shared Types** (`shared/`)
   - Protocol definitions
   - Genome structures
   - Communication types

## 🧬 Evolution Mechanics

Each creature has a genome with 5 genes (0.0 to 1.0):

- **Strength**: Combat power (high energy cost)
- **Speed**: Movement/escape ability (high energy cost)
- **Size**: Health/durability (high energy cost)
- **Efficiency**: Reduces energy consumption
- **Reproduction**: Breeding rate

The key insight: **No single strategy dominates**. High power requires high energy, creating an arms race where creatures must find optimal balances.

## 🚀 Quick Start

### Running Locally

#### Server
```bash
cd server
cargo run --release
```

Visit `http://localhost:8080` to see the dashboard.

#### Client
```bash
cd client
cargo run --release
```

Or set a custom server URL:
```bash
SERVER_URL=http://localhost:8080 cargo run --release
```

### Running with Docker

#### Build Images
```bash
# Server
docker build -f server/Dockerfile -t evo-islands-server .

# Client
docker build -f client/Dockerfile -t evo-islands-client .
```

#### Run Containers
```bash
# Start server
docker run -p 8080:8080 evo-islands-server

# Start client (connects to default server)
docker run evo-islands-client

# Or specify custom server
docker run -e SERVER_URL=http://your-server:8080 evo-islands-client
```

### Deploying to Kubernetes

```bash
# Deploy server
kubectl apply -f k8s/server-deployment.yaml

# Deploy clients (adjust replicas as needed)
kubectl apply -f k8s/client-deployment.yaml

# Scale clients
kubectl scale deployment evo-islands-client --replicas=10
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p shared
cargo test -p sim
cargo test -p server
cargo test -p client

# Run with logging
RUST_LOG=debug cargo test --all
```

## 📊 Monitoring

### Web Dashboard

Navigate to your server URL (e.g., `https://evo-islands.rackspace.koski.co`) to see:

- Active client count
- Total work units completed
- Total generations simulated
- Gene pool size
- Top evolved creatures with visualizations
- Real-time stats updates

### Server Logs

```bash
# Docker
docker logs -f <container-id>

# Kubernetes
kubectl logs -f deployment/evo-islands-server
```

### Client Logs

```bash
# Docker
docker logs -f <container-id>

# Kubernetes
kubectl logs -f deployment/evo-islands-client
```

## 🔧 Configuration

### Server

- **Port**: 8080 (configurable in code)
- **Gene Pool Size**: 100 best genomes + 1000 historical
- **Work Assignment**: 100 generations, 50 creatures, 5% mutation rate

### Client

- **Server URL**: Set via `SERVER_URL` environment variable
- **Default**: `https://evo-islands.rackspace.koski.co`
- **Retry Logic**: 10-second delay on connection failure
- **Version Checking**: Exits on mismatch (Kubernetes will restart)

## 📈 Performance

### Resource Usage

**Server**:
- Memory: ~256MB typical, 512MB limit
- CPU: ~250m typical, 500m limit

**Client**:
- Memory: ~256MB typical, 512MB limit
- CPU: ~250m typical, 500m limit (adjust for faster evolution)

### Scaling

- **Horizontal**: Add more client replicas
- **Vertical**: Increase client CPU for faster simulations
- **Server**: Single instance handles hundreds of clients

## 🔐 Security

- HTTPS/TLS via Kubernetes Ingress
- No authentication (public contribution model)
- Version checking prevents outdated clients
- Resource limits prevent runaway processes

## 🛠️ Development

### Project Structure

```
evo-islands/
├── shared/          # Common types and protocol
├── sim/             # Evolution simulation engine
├── server/          # Server implementation
│   ├── src/
│   │   ├── main.rs
│   │   ├── server.rs
│   │   ├── gene_pool.rs
│   │   └── web.rs
│   └── static/      # Web UI
├── client/          # Client implementation
│   └── src/
│       ├── main.rs
│       └── client.rs
├── k8s/             # Kubernetes manifests
├── gha/             # GitHub Actions workflows
└── Cargo.toml       # Workspace configuration
```

### Adding Features

1. **New Gene**: Add to `shared/src/genes.rs`
2. **New API Endpoint**: Add to `server/src/server.rs`
3. **UI Enhancement**: Modify `server/static/index.html`
4. **Simulation Tuning**: Adjust `sim/src/island.rs`

## 🐛 Troubleshooting

### Client Can't Connect

```bash
# Check server URL
echo $SERVER_URL

# Test server health
curl https://evo-islands.rackspace.koski.co/api/stats

# Check DNS
nslookup evo-islands.rackspace.koski.co
```

### Version Mismatch

Clients automatically exit on version mismatch. In Kubernetes, they'll restart with the new image.

```bash
# Force update clients
kubectl rollout restart deployment/evo-islands-client
```

### Out of Memory

Increase resource limits in `k8s/client-deployment.yaml`:

```yaml
resources:
  limits:
    memory: "1Gi"
```

## 📝 License

MIT

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --all`
5. Format code: `cargo fmt --all`
6. Check lints: `cargo clippy --all`
7. Submit a pull request

## 🙏 Acknowledgments

- Inspired by Folding@Home's distributed computing model
- Built with Rust, Tokio, Axum, and Ratatui
- Evolution mechanics inspired by genetic algorithms research

## 📧 Contact

For questions or issues, please open a GitHub issue.

---

**Live Server**: https://evo-islands.rackspace.koski.co

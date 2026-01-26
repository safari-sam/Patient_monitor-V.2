# Smart Patient Room Monitor (IoT + Rust + FHIR)

A multi-sensor IoT system for longitudinal activity tracking, environmental safety, and clinical decision support in healthcare facilities.

---

## 📌 Project Overview
This project bridges the gap between simple emergency alarms and comprehensive patient monitoring. By leveraging a hybrid IoT architecture, it combines real-time sensor data with clinical logic to support physiotherapy, elderly care, and mental health monitoring.

The system captures Motion, Sound, and Temperature data, processes it via a high-performance Rust backend, standardizes it into HL7 FHIR R4 resources, and visualizes it on a real-time "Nurse Station" dashboard.

### 🏥 Clinical Use Cases
* Physiotherapy: Validates patient mobility targets via "Average Physical Activity" scores.
* Elderly Care: Monitors "longest still periods" for pressure ulcer prevention and detects wandering.
* Patient Safety: Detects potential falls using a multi-factor algorithm (Motion + Peak Audio Amplitude).
* Mental Health: Tracks circadian rhythm disruptions (e.g., reversed sleep-wake cycles).

🏗️ Technical Architecture

## 1. Hardware Layer (Perception)
* Microcontroller: Arduino Uno R3 acting as the sensor hub.
* Sensors:
    * PIR Motion: For presence and activity intensity.
    * Sound (KY-038): Implements interrupt-based 1000Hz sampling to capture transient impact sounds (solving standard polling limitations).
    * DHT11: For ambient room temperature monitoring.

### 2. Backend Layer (Rust & Actix)
* Framework: Built with Rust and Actix-web for memory safety and high concurrency.
* Concurrency Model: Uses a dedicated background thread for serial ingestion and an actor-based model for WebSocket broadcasting.
* Data Processing:
    * Parses raw CSV streams in real-time.
* Interoperability: Transforms all data into FHIR R4 Observation resources using LOINC (8310-5, 89020-2) and SNOMED CT (52821000) codes.
* Storage: PostgreSQL database with connection pooling for persistent history.

### 3. Frontend Layer (Visualization)
* Stack: Vanilla JavaScript & D3.js (v7).
* Features:
    * Real-time Streaming: Sub-100ms latency via WebSockets.
    * Longitudinal Charts: Visualizes "Rest" vs. "Active" periods over 24h timelines.
    * Audio Alerts: Uses the Web Audio API for distinct, professional warning tones to mitigate alarm fatigue.

## 🛠️ Setup & Deployment

### ☁️ GitHub Codespaces 
This project is fully containerized and includes a **Mock Mode**, allowing you to test the full software stack (Backend, Database, Frontend) without physical sensor hardware.

1.  **Launch Codespace:** Click the green **Code** button > **Codespaces** > **Create codespace on main**.
2.  **Run via Docker:**
    The repository includes a `docker-compose.yml` that orchestrates the Rust backend and PostgreSQL database.
    ```bash
    docker-compose up --build
    ```
3.  **Verify Mock Data:**
    * Since Codespaces cannot access local USB ports, the system automatically detects the environment and switches to `MOCK_MODE`[cite: 69].
    * You will see simulated sensor data (randomized temperature, motion, and sound spikes) streaming to the dashboard immediately.
4.  **Access Dashboard:**
    * Click the "Ports" tab in VS Code.
    * Open the forwarded address for **Port 8080** (or 8000, depending on your config) to view the live dashboard.

---
### Running the Frontend
Simply serve the `frontend` directory using any static file server or open `index.html` directly (backend must be running).

---

## 🧪 Testing & Quality Assurance

### Automated Testing
The project includes comprehensive test coverage:
* **Unit Tests**: 81 tests covering Serial Parser, FHIR Transformation, and Alert Logic
* **Integration Tests**: Database operations, API endpoints, and FHIR compliance
* **ML Service Tests**: Training data generation and model validation

Run tests:
```bash
cd monitor/backend
cargo test              # Run all tests
cargo test --lib        # Unit tests only
cargo test --test '*'   # Integration tests only
```

### Code Quality Tools

#### Format Code
```bash
cd monitor/backend
cargo fmt              # Auto-format all Rust code
cargo fmt --check      # Check formatting without changing files
```

#### Lint Code
```bash
cd monitor/backend
cargo clippy           # Run Clippy linter
cargo clippy --fix     # Auto-fix issues where possible
```

#### Security Audit
```bash
cd monitor/backend
cargo install cargo-audit
cargo audit            # Check for known security vulnerabilities
```

---

## 🔄 CI/CD Pipeline

### GitHub Actions Workflow
Every push and pull request triggers an automated pipeline:

1. **🔍 Lint & Format** - Ensures code style consistency
2. **🧪 Run Tests** - Executes all unit and integration tests
3. **🔨 Build** - Compiles release binary
4. **🐳 Docker Build** - Builds and tests all containers
5. **🤖 ML Tests** - Validates ML service functionality
6. **🔒 Security Scan** - Checks for vulnerabilities

View pipeline status: **Actions** tab on GitHub

### Git Hooks

Install pre-commit hooks to catch issues before pushing:

```bash
# One-time setup
chmod +x setup-hooks.sh
./setup-hooks.sh
```

The pre-commit hook automatically runs:
- ✅ Code formatting check (`cargo fmt --check`)
- ✅ Linter (`cargo clippy`)
- ✅ Unit tests (`cargo test --lib`)
- ✅ Secret detection (prevents hardcoded passwords)

**Skip hooks** (not recommended):
```bash
git commit --no-verify
```

---

## 📝 Development Workflow

### Making Changes

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and test locally
   ```bash
   cd monitor/backend
   cargo fmt          # Format code
   cargo clippy       # Check for issues
   cargo test         # Run tests
   ```

3. **Commit changes** (pre-commit hook runs automatically)
   ```bash
   git add .
   git commit -m "feat: your descriptive message"
   ```

4. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   - CI/CD pipeline runs automatically
   - All checks must pass before merge

### Configuration Files

- **`.github/workflows/ci.yml`** - GitHub Actions CI/CD pipeline
- **`hooks/pre-commit`** - Git pre-commit hook
- **`monitor/backend/rustfmt.toml`** - Code formatting rules
- **`monitor/backend/clippy.toml`** - Linter configuration

---

Author: Samuel Safari Onyango  

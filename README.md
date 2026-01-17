# Smart Patient Room Monitor

A real-time patient monitoring system built with Rust, PostgreSQL, and WebSockets. Features include vital signs monitoring, fall detection, sleep analysis, and healthcare provider authentication.

## Features

- 🏥 **Real-time Monitoring**: Live sensor data via WebSockets
- 📊 **FHIR Compliance**: Medical data standardization
- 🔐 **Authentication**: Healthcare provider login with role-based access
- 📈 **Analytics**: Sleep patterns, hourly trends, period analysis
- 🚨 **Alert System**: Fall detection and inactivity monitoring
- 🎨 **Modern UI**: Interactive dashboard with D3.js visualizations

## Tech Stack

- **Backend**: Rust (Actix-web)
- **Database**: PostgreSQL
- **Frontend**: Vanilla JavaScript, D3.js
- **Authentication**: JWT tokens with bcrypt password hashing
- **Deployment**: Docker & Docker Compose

## Quick Start with Docker

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd Rust.patient_monitor
   ```

2. **Start with Docker Compose**
   ```bash
   cd monitor
   docker-compose up --build
   ```

3. **Access the application**
   - Open: http://localhost:8080
   - Create an account and login
   - Start monitoring!

## Running in GitHub Codespaces

This project is fully configured to run in GitHub Codespaces:

1. Click "Code" → "Codespaces" → "Create codespace"
2. Wait for the environment to build
3. Run: `docker-compose up --build`
4. Open the forwarded port 8080 in your browser

## Healthcare Roles

The system supports four healthcare provider roles:
- 👨‍⚕️ **Physician/Doctor**
- 👩‍⚕️ **Nurse**
- 🏃‍♂️ **Physiotherapist**
- 🤝 **Caretaker/Care Assistant**

## Architecture

```
monitor/
├── backend/
│   ├── src/
│   │   ├── main.rs          # Application entry point
│   │   ├── api.rs           # REST API endpoints
│   │   ├── auth.rs          # Authentication & JWT
│   │   ├── db.rs            # Database operations
│   │   ├── fhir.rs          # FHIR data structures
│   │   ├── serial.rs        # Sensor data simulation
│   │   └── websocket.rs     # Real-time updates
│   └── frontend/
│       ├── index.html       # Dashboard
│       ├── login.html       # Authentication page
│       ├── app.js           # Frontend logic
│       └── style.css        # Styling
├── docker-compose.yml       # Docker orchestration
└── Dockerfile              # Container configuration
```

## Environment Variables

The application uses these environment variables (configured in Dockerfile):

- `HOST`: Server host (default: 0.0.0.0)
- `PORT`: Server port (default: 8080)
- `DB_HOST`: PostgreSQL host (default: db)
- `DB_PORT`: PostgreSQL port (default: 5432)
- `DB_USER`: Database user (default: postgres)
- `DB_PASSWORD`: Database password (default: postgres)
- `DB_NAME`: Database name (default: patient_monitor)
- `MOCK_MODE`: Use mock sensor data (default: true)

## API Endpoints

### Authentication
- `POST /api/auth/signup` - Register new user
- `POST /api/auth/login` - Login and get JWT token
- `GET /api/auth/verify` - Verify token validity
- `GET /api/auth/me` - Get current user info

### Monitoring
- `GET /api/observations` - List observations
- `GET /api/observations/latest` - Latest reading
- `GET /api/observations/:id` - Get specific observation
- `GET /api/summary` - Overall statistics
- `GET /api/analysis/sleep` - Sleep analysis
- `GET /api/analysis/hourly` - Hourly trends
- `WS /ws` - WebSocket for real-time updates

## Development

### Local Development (without Docker)

1. Install Rust: https://rustup.rs/
2. Install PostgreSQL
3. Set environment variables
4. Run:
   ```bash
   cd monitor/backend
   cargo run
   ```

### Building for Production

```bash
cd monitor
docker-compose up --build -d
```

## License

MIT License

## Author

Your Name

# ML System Integration - Smart Patient Room Monitor

## Overview
This ML system provides real-time activity classification for patient monitoring using sensor data (temperature, motion, sound). The system uses a Random Forest classifier trained on FHIR-compliant synthetic data.

## Activity Classifications

The ML model classifies patient activity into 6 categories:

| Class | Risk Level | Description | Icon |
|-------|-----------|-------------|------|
| **SLEEPING** | Low | Deep rest during night hours | 😴 |
| **RESTING** | Low | Light rest, minimal activity | 🛋️ |
| **ACTIVE** | Normal | Normal daily movement | 🚶 |
| **RESTLESS** | Medium | Agitated, frequent movement | 😰 |
| **FALL_RISK** | High | Patterns indicating potential fall | ⚠️ |
| **FALL_DETECTED** | Critical | Fall event occurred | 🚨 |

## Architecture

```
┌─────────────────┐
│  Arduino        │
│  Sensors        │
└────────┬────────┘
         │
         v
┌─────────────────┐     ┌──────────────┐
│  Rust Backend   │────>│  PostgreSQL  │
│  (Port 8080)    │     │  Database    │
└────────┬────────┘     └──────────────┘
         │
         │ HTTP
         v
┌─────────────────┐
│  Python ML      │
│  Service        │
│  (Port 5001)    │
└─────────────────┘
```

## Directory Structure

```
monitor/
├── backend/
│   ├── src/
│   │   ├── ml_client.rs      # HTTP client for ML service
│   │   ├── ml_api.rs         # API endpoints (/api/ml/*)
│   │   └── main.rs           # Updated with ML integration
│   ├── frontend/
│   │   ├── index.html        # Updated with ML card
│   │   ├── app.js            # Updated with ML functions
│   │   └── style.css         # ML styles added
│   └── Cargo.toml            # Added reqwest dependency
├── ml_service/
│   ├── training_data/
│   │   └── generate_data.py  # Generates FHIR training data
│   ├── train_model.py        # Trains Random Forest model
│   ├── ml_service.py         # Flask API server
│   ├── requirements.txt      # Python dependencies
│   └── Dockerfile            # ML service container
└── docker-compose.yml        # Added ml-service container
```

## API Endpoints

### ML Service (Python - Port 5001)

- `GET  /health` - Check service health
- `GET  /model/info` - Get model metadata
- `POST /predict` - Single prediction
- `POST /predict/batch` - Batch predictions
- `POST /fhir/classify` - FHIR observation classification

### Backend API (Rust - Port 8080)

- `GET  /api/ml/health` - Check ML service availability
- `POST /api/ml/classify` - Classify sensor reading
- `GET  /api/ml/model/info` - Get model information

## Data Flow

1. **Sensor Reading** → Arduino sends data to Rust backend
2. **Storage** → Rust saves to PostgreSQL
3. **ML Classification** → Rust calls Python ML service
4. **Prediction** → ML service returns activity classification
5. **WebSocket Broadcast** → Real-time update to dashboard
6. **Display** → Frontend shows classification with confidence

## ML Model Training

### 1. Generate Training Data

```bash
cd monitor/ml_service
python training_data/generate_data.py
```

Generates:
- `training_data/training_data.csv` - 5000 labeled samples
- `training_data/fhir_observations.json` - FHIR Bundle

### 2. Train Model

```bash
python train_model.py
```

Outputs:
- `models/activity_classifier.joblib` - Trained Random Forest
- `models/label_encoder.joblib` - Label encoder
- `models/scaler.joblib` - Feature scaler
- `models/model_metadata.json` - Model info & metrics

### 3. Model Performance

Expected metrics:
- **Accuracy**: ~85-90%
- **F1 Score**: ~0.85
- **Cross-validation**: 5-fold CV with mean accuracy ~87%

## Running the System

### With Docker Compose (Recommended)

```bash
cd monitor
docker-compose up --build
```

This will:
1. Start PostgreSQL database
2. Build and train ML model
3. Start ML service on port 5001
4. Start Rust backend on port 8080

### Access Points

- **Dashboard**: http://localhost:8080
- **ML Service**: http://localhost:5001
- **API Health**: http://localhost:8080/api/health
- **ML Health**: http://localhost:8080/api/ml/health

### Environment Variables

Add to `.env` or docker-compose environment:

```env
# ML Service
ML_SERVICE_URL=http://ml-service:5001

# Database
DB_PASSWORD=your_secure_password

# Authentication
JWT_SECRET=your_jwt_secret
```

## Frontend Integration

The dashboard now includes a **ML Classification Card** that shows:

1. **Current Activity** - Large icon and class name
2. **Confidence Level** - Percentage confidence
3. **Risk Badge** - Color-coded risk level
4. **Confidence Bars** - Top 5 predictions with percentages
5. **History** - Recent classifications with timestamps

### Real-time Updates

Every new sensor reading automatically:
1. Gets classified by the ML model
2. Updates the activity display
3. Shows confidence scores
4. Adds to classification history

## Testing ML Service

### Health Check

```bash
curl http://localhost:5001/health
```

### Single Prediction

```bash
curl -X POST http://localhost:5001/predict \
  -H "Content-Type: application/json" \
  -d '{
    "temperature": 23.5,
    "motion_level": 45,
    "sound_level": 120,
    "hour_of_day": 14,
    "is_night": 0,
    "motion_trend": 5.2
  }'
```

### Via Rust Backend

```bash
curl -X POST http://localhost:8080/api/ml/classify \
  -H "Content-Type: application/json" \
  -d '{
    "temperature": 23.5,
    "motion_level": 45,
    "sound_level": 120
  }'
```

## Feature Engineering

The model uses 6 features:

1. **temperature** - Room temperature (°C)
2. **motion_level** - Motion sensor value (0-100)
3. **sound_level** - Sound level (0-255)
4. **hour_of_day** - Hour (0-23)
5. **is_night** - Binary flag (1 if 22:00-06:00)
6. **motion_trend** - Change from previous reading

## FHIR Compliance

Training data includes FHIR Observation resources with:

- **LOINC 8310-5**: Body temperature
- **SNOMED 52821000**: Activity/motion
- **LOINC 89020-2**: Sound level

## Troubleshooting

### ML Service Not Starting

Check Docker logs:
```bash
docker logs ml-service
```

Ensure model is trained:
```bash
docker exec -it ml-service ls -la models/
```

### Classification Not Showing

1. Check ML service health: `/api/ml/health`
2. Check browser console for errors
3. Verify WebSocket connection is active
4. Check that sensor data is being received

### Low Accuracy

1. Retrain model with more data
2. Adjust `CLASS_DISTRIBUTION` in `generate_data.py`
3. Tune hyperparameters in `train_model.py`

## Future Enhancements

- [ ] Model retraining with real patient data
- [ ] Multi-patient support with patient-specific models
- [ ] Anomaly detection for unusual patterns
- [ ] Integration with hospital alert systems
- [ ] Export predictions to FHIR server
- [ ] A/B testing different ML models
- [ ] Model performance monitoring dashboard

## Dependencies

### Python (ML Service)
- Flask 3.0.0 - Web framework
- scikit-learn 1.3.2 - ML library
- pandas 2.1.3 - Data processing
- numpy 1.26.2 - Numerical computing

### Rust (Backend)
- reqwest 0.11 - HTTP client for ML service
- actix-web 4 - Web framework
- tokio - Async runtime

## License

Part of Smart Patient Room Monitor project.

## Support

For issues or questions:
1. Check logs: `docker-compose logs ml-service`
2. Verify model files exist in `ml_service/models/`
3. Test ML service independently: `curl http://localhost:5001/health`

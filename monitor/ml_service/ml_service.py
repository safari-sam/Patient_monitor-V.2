"""
============================================================================
Smart Patient Room Monitor - ML Prediction Service (Flask API)
============================================================================
A REST API service that provides real-time activity classification
predictions based on sensor data.

Endpoints:
- POST /predict         - Single prediction
- POST /predict/batch   - Batch predictions
- GET  /health          - Health check
- GET  /model/info      - Model metadata
- POST /retrain         - Trigger model retraining (with new data)

This service is called by the Rust backend for real-time classification.
============================================================================
"""

from flask import Flask, request, jsonify
from flask_cors import CORS
import joblib
import numpy as np
import logging
import os
import threading
from datetime import datetime
from typing import Dict, List, Optional

# ============================================================================
# CONFIGURATION
# ============================================================================

MODEL_DIR = "models"
MODEL_PATH = f"{MODEL_DIR}/activity_classifier.joblib"
ENCODER_PATH = f"{MODEL_DIR}/label_encoder.joblib"
SCALER_PATH = f"{MODEL_DIR}/scaler.joblib"
METADATA_PATH = f"{MODEL_DIR}/model_metadata.json"

# Features expected by the model
FEATURES = [
    "temperature",
    "motion_level",
    "sound_level", 
    "hour_of_day",
    "is_night",
    "motion_trend"
]

# ============================================================================
# FLASK APP SETUP
# ============================================================================

app = Flask(__name__)
CORS(app)  # Allow cross-origin requests from Rust backend

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# ============================================================================
# MODEL LOADING
# ============================================================================

class ModelManager:
    """Manages ML model loading and predictions."""
    
    def __init__(self):
        self.model = None
        self.label_encoder = None
        self.scaler = None
        self.metadata = None
        self.is_loaded = False
        self.load_lock = threading.Lock()
    
    def load_model(self) -> bool:
        """Load the trained model and preprocessing objects."""
        with self.load_lock:
            try:
                logger.info("Loading ML model...")
                
                # Load model
                self.model = joblib.load(MODEL_PATH)
                logger.info(f"✓ Model loaded from {MODEL_PATH}")
                
                # Load label encoder
                self.label_encoder = joblib.load(ENCODER_PATH)
                logger.info(f"✓ Label encoder loaded")
                
                # Load scaler
                self.scaler = joblib.load(SCALER_PATH)
                logger.info(f"✓ Scaler loaded")
                
                # Load metadata
                import json
                with open(METADATA_PATH, 'r') as f:
                    self.metadata = json.load(f)
                logger.info(f"✓ Metadata loaded")
                
                self.is_loaded = True
                logger.info("✅ ML model ready!")
                return True
                
            except Exception as e:
                logger.error(f"❌ Failed to load model: {e}")
                self.is_loaded = False
                return False
    
    def predict(self, features: Dict) -> Dict:
        """Make a single prediction."""
        if not self.is_loaded:
            raise RuntimeError("Model not loaded")
        
        # Prepare features
        feature_values = [features.get(f, 0) for f in FEATURES]
        X = np.array([feature_values])
        
        # Scale features
        X_scaled = self.scaler.transform(X)
        
        # Predict
        prediction = self.model.predict(X_scaled)[0]
        probabilities = self.model.predict_proba(X_scaled)[0]
        
        # Decode label
        activity_class = self.label_encoder.inverse_transform([prediction])[0]
        
        # Get confidence scores for all classes
        confidence_scores = {
            self.label_encoder.inverse_transform([i])[0]: float(prob)
            for i, prob in enumerate(probabilities)
        }
        
        return {
            "activity_class": activity_class,
            "confidence": float(max(probabilities)),
            "confidence_scores": confidence_scores
        }
    
    def predict_batch(self, features_list: List[Dict]) -> List[Dict]:
        """Make batch predictions."""
        if not self.is_loaded:
            raise RuntimeError("Model not loaded")
        
        # Prepare features
        X = np.array([
            [f.get(feat, 0) for feat in FEATURES]
            for f in features_list
        ])
        
        # Scale features
        X_scaled = self.scaler.transform(X)
        
        # Predict
        predictions = self.model.predict(X_scaled)
        probabilities = self.model.predict_proba(X_scaled)
        
        results = []
        for i, (pred, probs) in enumerate(zip(predictions, probabilities)):
            activity_class = self.label_encoder.inverse_transform([pred])[0]
            results.append({
                "index": i,
                "activity_class": activity_class,
                "confidence": float(max(probs))
            })
        
        return results


# Global model manager
model_manager = ModelManager()

# ============================================================================
# API ENDPOINTS
# ============================================================================

@app.route('/health', methods=['GET'])
def health_check():
    """Health check endpoint."""
    return jsonify({
        "status": "healthy",
        "model_loaded": model_manager.is_loaded,
        "timestamp": datetime.now().isoformat()
    })


@app.route('/model/info', methods=['GET'])
def model_info():
    """Get model metadata."""
    if not model_manager.is_loaded:
        return jsonify({"error": "Model not loaded"}), 503
    
    return jsonify({
        "model_loaded": True,
        "metadata": model_manager.metadata,
        "classes": list(model_manager.label_encoder.classes_),
        "features": FEATURES
    })


@app.route('/predict', methods=['POST'])
def predict():
    """
    Make a single prediction.
    
    Expected JSON body:
    {
        "temperature": 23.5,
        "motion_level": 45,
        "sound_level": 120,
        "hour_of_day": 14,
        "is_night": 0,
        "motion_trend": 5.2
    }
    
    Returns:
    {
        "activity_class": "ACTIVE",
        "confidence": 0.87,
        "confidence_scores": {...}
    }
    """
    if not model_manager.is_loaded:
        return jsonify({"error": "Model not loaded"}), 503
    
    try:
        data = request.get_json()
        prediction = model_manager.predict(data)
        return jsonify(prediction)
    except Exception as e:
        logger.error(f"Prediction error: {e}")
        return jsonify({"error": str(e)}), 500


@app.route('/predict/batch', methods=['POST'])
def predict_batch():
    """
    Make batch predictions.
    
    Expected JSON body:
    {
        "readings": [
            {"temperature": 23.5, "motion_level": 45, ...},
            {"temperature": 22.1, "motion_level": 10, ...}
        ]
    }
    """
    if not model_manager.is_loaded:
        return jsonify({"error": "Model not loaded"}), 503
    
    try:
        data = request.get_json()
        readings = data.get('readings', [])
        predictions = model_manager.predict_batch(readings)
        return jsonify({"predictions": predictions})
    except Exception as e:
        logger.error(f"Batch prediction error: {e}")
        return jsonify({"error": str(e)}), 500


@app.route('/classify', methods=['POST'])
def classify():
    """Alias for /predict (for compatibility)."""
    return predict()


@app.route('/retrain', methods=['POST'])
def retrain():
    """Trigger model retraining with new data."""
    # This is a placeholder - implement if needed
    return jsonify({
        "message": "Retraining not implemented yet",
        "status": "not_implemented"
    }), 501


# ============================================================================
# FHIR-COMPATIBLE ENDPOINT
# ============================================================================

@app.route('/fhir/classify', methods=['POST'])
def fhir_classify():
    """
    Accept FHIR Observation and return classification.
    
    Extracts sensor values from FHIR Observation components.
    """
    if not model_manager.is_loaded:
        return jsonify({"error": "Model not loaded"}), 503
    
    try:
        observation = request.get_json()
        
        # Extract values from FHIR components
        features = {}
        for component in observation.get('component', []):
            code = component.get('code', {})
            # Temperature
            if '8310-5' in str(code):
                features['temperature'] = component.get('valueQuantity', {}).get('value', 0)
            # Motion
            elif '52821000' in str(code):
                features['motion_level'] = component.get('valueInteger', 0)
            # Sound
            elif '89020-2' in str(code):
                features['sound_level'] = component.get('valueInteger', 0)
        
        # Add time-based features
        from datetime import datetime
        now = datetime.now()
        features['hour_of_day'] = now.hour
        features['is_night'] = 1 if (now.hour >= 22 or now.hour < 6) else 0
        features['motion_trend'] = 0  # Would need history
        
        prediction = model_manager.predict(features)
        return jsonify(prediction)
        
    except Exception as e:
        logger.error(f"FHIR classification error: {e}")
        return jsonify({"error": str(e)}), 500


# ============================================================================
# STARTUP
# ============================================================================

@app.before_request
def ensure_model_loaded():
    """Ensure model is loaded before handling requests."""
    if not model_manager.is_loaded and request.endpoint != 'health_check':
        model_manager.load_model()


def initialize():
    """Initialize the service."""
    logger.info("="*60)
    logger.info("🏥 Smart Patient Room Monitor - ML Service")
    logger.info("="*60)
    
    # Load model
    if model_manager.load_model():
        logger.info("✅ Service initialization complete")
    else:
        logger.warning("⚠️  Service started but model not loaded")


# ============================================================================
# MAIN
# ============================================================================

if __name__ == "__main__":
    initialize()
    
    # Run Flask server
    port = int(os.environ.get("ML_PORT", 5001))
    
    logger.info(f"🚀 Starting ML service on port {port}...")
    
    app.run(
        host="0.0.0.0",
        port=port,
        debug=os.environ.get("DEBUG", "false").lower() == "true"
    )

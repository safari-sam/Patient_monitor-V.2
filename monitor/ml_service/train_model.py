"""
============================================================================
Smart Patient Room Monitor - ML Model Training
============================================================================
Trains a classification model to categorize patient activity based on
sensor data (motion, sound, temperature).

Models:
- Decision Tree (simple, interpretable)
- Random Forest (better accuracy)

Output:
- Trained model (model.joblib)
- Model metrics and evaluation
- Feature importance analysis
============================================================================
"""

import pandas as pd
import numpy as np
import joblib
import os
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.tree import DecisionTreeClassifier
from sklearn.ensemble import RandomForestClassifier
from sklearn.preprocessing import LabelEncoder, StandardScaler
from sklearn.metrics import (
    classification_report, 
    confusion_matrix, 
    accuracy_score,
    f1_score
)
import json
import warnings
warnings.filterwarnings('ignore')
from datetime import datetime

# ============================================================================
# CONFIGURATION
# ============================================================================

DATA_FILE = "training_data/training_data.csv"
MODEL_OUTPUT_DIR = "models"
MODEL_TYPE = "random_forest"  # "decision_tree" or "random_forest"

# Features to use for training
FEATURES = [
    "temperature",
    "motion_level", 
    "sound_level",
    "hour_of_day",
    "is_night",
    "motion_trend"
]

# Target column
TARGET = "activity_class"

# ============================================================================
# DATA LOADING AND PREPROCESSING
# ============================================================================

def load_data(filepath: str) -> pd.DataFrame:
    """Load and validate training data."""
    print(f"📂 Loading data from {filepath}...")
    
    df = pd.read_csv(filepath)
    print(f"   Loaded {len(df)} samples")
    
    # Validate required columns
    required_cols = FEATURES + [TARGET]
    missing = [col for col in required_cols if col not in df.columns]
    if missing:
        raise ValueError(f"Missing columns: {missing}")
    
    return df


def preprocess_data(df: pd.DataFrame) -> tuple:
    """Preprocess data for training."""
    print("🔧 Preprocessing data...")
    
    # Extract features and target
    X = df[FEATURES].copy()
    y = df[TARGET].copy()
    
    # Handle missing values
    X = X.fillna(X.mean())
    
    # Encode target labels
    label_encoder = LabelEncoder()
    y_encoded = label_encoder.fit_transform(y)
    
    # Scale features
    scaler = StandardScaler()
    X_scaled = scaler.fit_transform(X)
    X_scaled = pd.DataFrame(X_scaled, columns=FEATURES)
    
    print(f"   Features: {FEATURES}")
    print(f"   Classes: {list(label_encoder.classes_)}")
    
    return X_scaled, y_encoded, label_encoder, scaler


# ============================================================================
# MODEL TRAINING
# ============================================================================

def train_decision_tree(X_train, y_train) -> DecisionTreeClassifier:
    """Train a Decision Tree classifier."""
    print("🌳 Training Decision Tree...")
    
    model = DecisionTreeClassifier(
        max_depth=10,
        min_samples_split=5,
        min_samples_leaf=2,
        random_state=42
    )
    model.fit(X_train, y_train)
    
    return model


def train_random_forest(X_train, y_train) -> RandomForestClassifier:
    """Train a Random Forest classifier."""
    print("🌲 Training Random Forest...")
    
    model = RandomForestClassifier(
        n_estimators=100,
        max_depth=15,
        min_samples_split=5,
        min_samples_leaf=2,
        random_state=42,
        n_jobs=-1  # Use all CPU cores
    )
    model.fit(X_train, y_train)
    
    return model


# ============================================================================
# MODEL EVALUATION
# ============================================================================

def evaluate_model(model, X_test, y_test, label_encoder) -> dict:
    """Evaluate model performance."""
    print("\n📊 Evaluating model...")
    
    # Predictions
    y_pred = model.predict(X_test)
    
    # Metrics
    accuracy = accuracy_score(y_test, y_pred)
    f1 = f1_score(y_test, y_pred, average='weighted')
    
    # Classification report
    class_names = label_encoder.classes_
    report = classification_report(y_test, y_pred, target_names=class_names)
    
    # Confusion matrix
    cm = confusion_matrix(y_test, y_pred)
    
    print("\n" + "="*60)
    print("MODEL EVALUATION RESULTS")
    print("="*60)
    print(f"\n✅ Accuracy: {accuracy:.4f} ({accuracy*100:.1f}%)")
    print(f"✅ F1 Score: {f1:.4f}")
    print("\n📋 Classification Report:")
    print(report)
    
    print("\n📉 Confusion Matrix:")
    print(f"   Classes: {list(class_names)}")
    print(cm)
    
    return {
        "accuracy": accuracy,
        "f1_score": f1,
        "classification_report": report,
        "confusion_matrix": cm.tolist()
    }


def analyze_feature_importance(model, feature_names: list) -> dict:
    """Analyze and display feature importance."""
    print("\n🔍 Feature Importance:")
    print("-" * 40)
    
    importances = model.feature_importances_
    indices = np.argsort(importances)[::-1]
    
    importance_dict = {}
    for i, idx in enumerate(indices):
        importance_dict[feature_names[idx]] = float(importances[idx])
        print(f"{i+1}. {feature_names[idx]:15s}: {importances[idx]:.4f}")
    
    return importance_dict


# ============================================================================
# MODEL SAVING
# ============================================================================

def save_model(model, label_encoder, scaler, metrics: dict, importance: dict):
    """Save trained model and metadata."""
    os.makedirs(MODEL_OUTPUT_DIR, exist_ok=True)
    
    # Save model
    model_path = f"{MODEL_OUTPUT_DIR}/activity_classifier.joblib"
    joblib.dump(model, model_path)
    print(f"\n💾 Model saved to: {model_path}")
    
    # Save label encoder
    encoder_path = f"{MODEL_OUTPUT_DIR}/label_encoder.joblib"
    joblib.dump(label_encoder, encoder_path)
    print(f"💾 Label encoder saved to: {encoder_path}")
    
    # Save scaler
    scaler_path = f"{MODEL_OUTPUT_DIR}/scaler.joblib"
    joblib.dump(scaler, scaler_path)
    print(f"💾 Scaler saved to: {scaler_path}")
    
    # Save metadata
    metadata = {
        "model_type": MODEL_TYPE,
        "features": FEATURES,
        "classes": list(label_encoder.classes_),
        "metrics": metrics,
        "feature_importance": importance,
        "trained_at": datetime.now().isoformat()
    }
    
    metadata_path = f"{MODEL_OUTPUT_DIR}/model_metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    print(f"💾 Metadata saved to: {metadata_path}")


# ============================================================================
# CROSS-VALIDATION
# ============================================================================

def cross_validate_model(model, X, y, cv=5):
    """Perform cross-validation."""
    print(f"\n🔄 Performing {cv}-fold cross-validation...")
    scores = cross_val_score(model, X, y, cv=cv, scoring='accuracy')
    print(f"   Cross-validation scores: {scores}")
    print(f"   Mean accuracy: {scores.mean():.4f} (+/- {scores.std() * 2:.4f})")
    return scores


# ============================================================================
# MAIN TRAINING PIPELINE
# ============================================================================

def main():
    print("="*60)
    print("🏥 Smart Patient Room Monitor - ML Model Training")
    print("="*60)
    
    # Load data
    df = load_data(DATA_FILE)
    
    # Preprocess
    X, y, label_encoder, scaler = preprocess_data(df)
    
    # Split data
    print("\n📊 Splitting data (80% train, 20% test)...")
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=42, stratify=y
    )
    print(f"   Training samples: {len(X_train)}")
    print(f"   Testing samples: {len(X_test)}")
    
    # Train model
    if MODEL_TYPE == "decision_tree":
        model = train_decision_tree(X_train, y_train)
    else:
        model = train_random_forest(X_train, y_train)
    
    # Cross-validation
    cross_validate_model(model, X_train, y_train)
    
    # Evaluate
    metrics = evaluate_model(model, X_test, y_test, label_encoder)
    
    # Feature importance
    importance = analyze_feature_importance(model, FEATURES)
    
    # Save model
    save_model(model, label_encoder, scaler, metrics, importance)
    
    print("\n" + "="*60)
    print("✅ Model training complete!")
    print("="*60)


if __name__ == "__main__":
    main()

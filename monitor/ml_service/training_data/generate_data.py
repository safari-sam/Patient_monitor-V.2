"""
============================================================================
Smart Patient Room Monitor - Training Data Generator
============================================================================
Generates realistic, FHIR-compliant sensor data for ML training.

FHIR Mappings:
- Temperature: LOINC 8310-5 (Body temperature)
- Motion: SNOMED 52821000 (Activity)  
- Sound: LOINC 89020-2 (Sound level)

Activity Classifications:
- SLEEPING: Deep rest during night hours
- RESTING: Light rest, minimal activity
- ACTIVE: Normal daily movement
- RESTLESS: Agitated, frequent movement
- FALL_RISK: Patterns indicating potential fall
- FALL_DETECTED: Fall event occurred
============================================================================
"""

import pandas as pd
import numpy as np
import random
import json
import os
from datetime import datetime, timedelta
from typing import List, Dict, Tuple
import uuid

# ============================================================================
# CONFIGURATION
# ============================================================================

NUM_SAMPLES = 5000  # Total training samples
OUTPUT_DIR = "training_data"

# Activity class distribution (realistic hospital setting)
CLASS_DISTRIBUTION = {
    "SLEEPING": 0.25,      # 25% - Night hours mostly
    "RESTING": 0.30,       # 30% - Common during day
    "ACTIVE": 0.25,        # 25% - Normal activity
    "RESTLESS": 0.10,      # 10% - Some agitation
    "FALL_RISK": 0.07,     # 7% - Warning signs
    "FALL_DETECTED": 0.03  # 3% - Actual falls (rare)
}

# ============================================================================
# SENSOR PATTERNS FOR EACH ACTIVITY CLASS
# ============================================================================

ACTIVITY_PATTERNS = {
    "SLEEPING": {
        "motion_range": (0, 15),
        "sound_range": (20, 60),
        "temp_range": (20.0, 23.0),
        "hour_weights": {
            "night": 0.8,    # 22:00 - 06:00 (80% chance)
            "day": 0.2       # 06:00 - 22:00 (20% chance)
        },
        "motion_variance": 5,
        "sound_variance": 10
    },
    "RESTING": {
        "motion_range": (10, 30),
        "sound_range": (40, 80),
        "temp_range": (21.0, 24.0),
        "hour_weights": {
            "night": 0.3,
            "day": 0.7
        },
        "motion_variance": 8,
        "sound_variance": 15
    },
    "ACTIVE": {
        "motion_range": (40, 75),
        "sound_range": (70, 130),
        "temp_range": (22.0, 26.0),
        "hour_weights": {
            "night": 0.1,
            "day": 0.9
        },
        "motion_variance": 15,
        "sound_variance": 25
    },
    "RESTLESS": {
        "motion_range": (50, 85),
        "sound_range": (90, 160),
        "temp_range": (23.0, 27.0),
        "hour_weights": {
            "night": 0.4,
            "day": 0.6
        },
        "motion_variance": 25,
        "sound_variance": 35
    },
    "FALL_RISK": {
        "motion_range": (60, 90),
        "sound_range": (100, 180),
        "temp_range": (22.0, 26.0),
        "hour_weights": {
            "night": 0.5,
            "day": 0.5
        },
        "motion_variance": 30,
        "sound_variance": 40
    },
    "FALL_DETECTED": {
        "motion_range": (85, 100),
        "sound_range": (150, 250),
        "temp_range": (22.0, 26.0),
        "hour_weights": {
            "night": 0.4,
            "day": 0.6
        },
        "motion_variance": 10,  # Sudden spike then low
        "sound_variance": 30
    }
}

# ============================================================================
# DATA GENERATION FUNCTIONS
# ============================================================================

def get_hour_for_activity(activity: str) -> int:
    """Generate appropriate hour based on activity type."""
    pattern = ACTIVITY_PATTERNS[activity]
    weights = pattern["hour_weights"]
    
    if random.random() < weights["night"]:
        # Night hours: 22, 23, 0, 1, 2, 3, 4, 5
        return random.choice([22, 23, 0, 1, 2, 3, 4, 5])
    else:
        # Day hours: 6-21
        return random.randint(6, 21)


def generate_sensor_reading(activity: str, timestamp: datetime) -> Dict:
    """Generate a single sensor reading for given activity class."""
    pattern = ACTIVITY_PATTERNS[activity]
    
    # Base values from ranges
    motion_base = random.uniform(*pattern["motion_range"])
    sound_base = random.uniform(*pattern["sound_range"])
    temp_base = random.uniform(*pattern["temp_range"])
    
    # Add some variance
    motion = max(0, min(100, motion_base + random.gauss(0, pattern["motion_variance"] / 3)))
    sound = max(0, min(255, sound_base + random.gauss(0, pattern["sound_variance"] / 3)))
    temperature = round(temp_base + random.gauss(0, 0.5), 1)
    
    # Special handling for FALL_DETECTED - spike pattern
    if activity == "FALL_DETECTED":
        motion = random.uniform(85, 100)
        sound = random.uniform(180, 250)
    
    # Derived features
    hour = timestamp.hour
    is_night = 1 if (hour >= 22 or hour < 6) else 0
    
    # Calculate motion trend (would normally use previous readings)
    motion_trend = random.uniform(-20, 20)  # Simulated
    
    return {
        "id": str(uuid.uuid4()),
        "timestamp": timestamp.isoformat() + "Z",
        "temperature": round(temperature, 1),
        "motion_level": int(motion),
        "sound_level": int(sound),
        "hour_of_day": hour,
        "is_night": is_night,
        "motion_trend": round(motion_trend, 2),
        "activity_class": activity
    }


def generate_fhir_observation(reading: Dict, patient_id: str = "patient-001") -> Dict:
    """Convert sensor reading to FHIR Observation resource."""
    observation_id = str(uuid.uuid4())
    
    return {
        "resourceType": "Observation",
        "id": observation_id,
        "status": "final",
        "category": [{
            "coding": [{
                "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                "code": "vital-signs",
                "display": "Vital Signs"
            }]
        }],
        "code": {
            "coding": [{
                "system": "http://loinc.org",
                "code": "85353-1",
                "display": "Vital signs, weight, height, head circumference, oxygen saturation & BMI panel"
            }],
            "text": "Sensor Reading"
        },
        "subject": {
            "reference": f"Patient/{patient_id}"
        },
        "effectiveDateTime": reading["timestamp"],
        "issued": reading["timestamp"],
        "component": [
            {
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "8310-5",
                        "display": "Body temperature"
                    }]
                },
                "valueQuantity": {
                    "value": reading["temperature"],
                    "unit": "Cel",
                    "system": "http://unitsofmeasure.org",
                    "code": "Cel"
                }
            },
            {
                "code": {
                    "coding": [{
                        "system": "http://snomed.info/sct",
                        "code": "52821000",
                        "display": "Activity"
                    }]
                },
                "valueInteger": reading["motion_level"]
            },
            {
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "89020-2",
                        "display": "Sound level"
                    }]
                },
                "valueInteger": reading["sound_level"]
            },
            {
                "code": {
                    "text": "Activity Classification"
                },
                "valueString": reading["activity_class"]
            }
        ]
    }


def generate_dataset() -> Tuple[List[Dict], List[Dict]]:
    """Generate complete dataset with FHIR observations."""
    print("🏥 Generating Smart Patient Room Monitor Training Data")
    print("=" * 60)
    
    readings = []
    observations = []
    
    # Generate samples for each class
    for activity, percentage in CLASS_DISTRIBUTION.items():
        count = int(NUM_SAMPLES * percentage)
        print(f"Generating {count:5d} samples for {activity:15s} ({percentage*100:4.1f}%)")
        
        for _ in range(count):
            # Generate timestamp
            hour = get_hour_for_activity(activity)
            timestamp = datetime.now() - timedelta(
                days=random.randint(0, 30),
                hours=random.randint(0, 23),
                minutes=random.randint(0, 59)
            )
            timestamp = timestamp.replace(hour=hour)
            
            # Generate reading
            reading = generate_sensor_reading(activity, timestamp)
            readings.append(reading)
            
            # Generate FHIR observation
            observation = generate_fhir_observation(reading)
            observations.append(observation)
    
    # Shuffle
    combined = list(zip(readings, observations))
    random.shuffle(combined)
    readings, observations = zip(*combined)
    
    return list(readings), list(observations)


def save_csv(readings: List[Dict], filename: str):
    """Save readings to CSV."""
    df = pd.DataFrame(readings)
    # Reorder columns
    columns = ["id", "timestamp", "temperature", "motion_level", "sound_level", 
               "hour_of_day", "is_night", "motion_trend", "activity_class"]
    df = df[columns]
    df.to_csv(filename, index=False)
    print(f"💾 Saved CSV: {filename}")


def save_fhir_bundle(observations: List[Dict], filename: str):
    """Save FHIR observations as a Bundle."""
    bundle = {
        "resourceType": "Bundle",
        "type": "collection",
        "entry": [{"resource": obs} for obs in observations]
    }
    with open(filename, 'w') as f:
        json.dump(bundle, f, indent=2)
    print(f"💾 Saved FHIR Bundle: {filename}")


def print_statistics(readings: List[Dict]):
    """Print dataset statistics."""
    df = pd.DataFrame(readings)
    
    print("\n" + "=" * 60)
    print("📊 Dataset Statistics")
    print("=" * 60)
    print(f"Total samples: {len(readings)}")
    print(f"\nClass distribution:")
    print(df['activity_class'].value_counts().sort_index())
    
    print(f"\nFeature ranges:")
    print(f"  Temperature: {df['temperature'].min():.1f} - {df['temperature'].max():.1f} °C")
    print(f"  Motion:      {df['motion_level'].min()} - {df['motion_level'].max()}")
    print(f"  Sound:       {df['sound_level'].min()} - {df['sound_level'].max()}")
    
    print(f"\nTime distribution:")
    print(f"  Night (is_night=1): {(df['is_night'] == 1).sum()} ({(df['is_night'] == 1).sum()/len(df)*100:.1f}%)")
    print(f"  Day (is_night=0):   {(df['is_night'] == 0).sum()} ({(df['is_night'] == 0).sum()/len(df)*100:.1f}%)")


# ============================================================================
# MAIN
# ============================================================================

if __name__ == "__main__":
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    # Generate data
    readings, observations = generate_dataset()
    
    # Save files
    save_csv(readings, f"{OUTPUT_DIR}/training_data.csv")
    save_fhir_bundle(observations, f"{OUTPUT_DIR}/fhir_observations.json")
    
    # Print statistics
    print_statistics(readings)
    
    print("\n✅ Training data generation complete!")
    print(f"📁 Output directory: {OUTPUT_DIR}/")

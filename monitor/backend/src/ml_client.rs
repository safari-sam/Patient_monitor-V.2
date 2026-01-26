// ============================================================================
// ML Client for Rust Backend
// ============================================================================
// This module communicates with the Python ML service to get
// activity classifications for sensor readings.
//
// Usage:
//   let client = MLClient::new("http://localhost:5001");
//   let prediction = client.classify(&sensor_reading).await?;
// ============================================================================

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, debug};
use chrono::Timelike;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Input features for ML prediction
#[derive(Debug, Clone, Serialize)]
pub struct MLFeatures {
    pub temperature: f32,
    pub motion_level: i32,
    pub sound_level: i32,
    pub hour_of_day: i32,
    pub is_night: i32,
    pub motion_trend: f32,
}

/// ML prediction result
#[derive(Debug, Clone, Deserialize)]
pub struct MLPrediction {
    pub activity_class: String,
    pub confidence: f32,
    #[serde(default)]
    pub confidence_scores: std::collections::HashMap<String, f32>,
}

/// Batch prediction request
#[derive(Debug, Serialize)]
struct BatchRequest {
    readings: Vec<MLFeatures>,
}

/// Batch prediction response
#[derive(Debug, Deserialize)]
struct BatchResponse {
    predictions: Vec<BatchPrediction>,
}

#[derive(Debug, Deserialize)]
struct BatchPrediction {
    index: usize,
    activity_class: String,
    confidence: f32,
}

/// ML service health response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub timestamp: String,
}

/// Model info response
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelInfo {
    pub model_loaded: bool,
    pub classes: Vec<String>,
    pub features: Vec<String>,
}

// ============================================================================
// ML CLIENT
// ============================================================================

/// Client for communicating with ML prediction service
pub struct MLClient {
    base_url: String,
    client: Client,
}

impl MLClient {
    /// Create a new ML client
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            base_url: base_url.to_string(),
            client,
        }
    }
    
    /// Create ML client from environment variable
    pub fn from_env() -> Self {
        let url = std::env::var("ML_SERVICE_URL")
            .unwrap_or_else(|_| "http://ml-service:5001".to_string());
        Self::new(&url)
    }
    
    /// Check if ML service is healthy
    pub async fn health_check(&self) -> Result<HealthResponse, MLError> {
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| MLError::ConnectionError(e.to_string()))?;
        
        if response.status().is_success() {
            response.json().await
                .map_err(|e| MLError::ParseError(e.to_string()))
        } else {
            Err(MLError::ServiceError(
                format!("Health check failed: {}", response.status())
            ))
        }
    }
    
    /// Check if ML service is available
    pub async fn is_available(&self) -> bool {
        match self.health_check().await {
            Ok(health) => health.model_loaded,
            Err(_) => false,
        }
    }
    
    /// Get model information
    pub async fn get_model_info(&self) -> Result<ModelInfo, MLError> {
        let url = format!("{}/model/info", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| MLError::ConnectionError(e.to_string()))?;
        
        if response.status().is_success() {
            response.json().await
                .map_err(|e| MLError::ParseError(e.to_string()))
        } else {
            Err(MLError::ServiceError(
                format!("Failed to get model info: {}", response.status())
            ))
        }
    }
    
    /// Classify a single sensor reading
    pub async fn classify(&self, features: &MLFeatures) -> Result<MLPrediction, MLError> {
        let url = format!("{}/predict", self.base_url);
        
        debug!("Sending classification request to ML service");
        
        let response = self.client
            .post(&url)
            .json(features)
            .send()
            .await
            .map_err(|e| MLError::ConnectionError(e.to_string()))?;
        
        if response.status().is_success() {
            let prediction: MLPrediction = response.json().await
                .map_err(|e| MLError::ParseError(e.to_string()))?;
            
            info!("ML prediction: {} (confidence: {:.1}%)", 
                  prediction.activity_class, 
                  prediction.confidence * 100.0);
            
            Ok(prediction)
        } else {
            Err(MLError::ServiceError(format!("Classification failed: {}", response.status())))
        }
    }
    
    /// Classify multiple readings in a batch
    pub async fn classify_batch(&self, readings: Vec<MLFeatures>) -> Result<Vec<MLPrediction>, MLError> {
        let url = format!("{}/predict/batch", self.base_url);
        
        let request = BatchRequest { readings };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| MLError::ConnectionError(e.to_string()))?;
        
        if response.status().is_success() {
            let batch_response: BatchResponse = response.json().await
                .map_err(|e| MLError::ParseError(e.to_string()))?;
            
            // Convert batch predictions to MLPredictions
            let predictions: Vec<MLPrediction> = batch_response.predictions
                .into_iter()
                .map(|p| MLPrediction {
                    activity_class: p.activity_class,
                    confidence: p.confidence,
                    confidence_scores: std::collections::HashMap::new(),
                })
                .collect();
            
            Ok(predictions)
        } else {
            Err(MLError::ServiceError(format!("Batch classification failed: {}", response.status())))
        }
    }
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug)]
pub enum MLError {
    ConnectionError(String),
    ServiceError(String),
    ParseError(String),
}

impl std::fmt::Display for MLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MLError::ConnectionError(msg) => write!(f, "ML connection error: {}", msg),
            MLError::ServiceError(msg) => write!(f, "ML service error: {}", msg),
            MLError::ParseError(msg) => write!(f, "ML parse error: {}", msg),
        }
    }
}

impl std::error::Error for MLError {}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Convert a sensor reading to ML features
pub fn sensor_reading_to_features(
    temperature: f32,
    motion_level: i32,
    sound_level: i32,
    timestamp: &chrono::DateTime<chrono::Utc>,
    motion_trend: f32,
) -> MLFeatures {
    let hour = timestamp.hour() as i32;
    let is_night = if hour >= 22 || hour < 6 { 1 } else { 0 };
    
    MLFeatures {
        temperature,
        motion_level,
        sound_level,
        hour_of_day: hour,
        is_night,
        motion_trend,
    }
}

/// Get activity class as a user-friendly string
pub fn format_activity_class(class: &str) -> String {
    class.replace("_", " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Determine risk level from activity class
pub fn get_risk_level(activity_class: &str) -> &'static str {
    match activity_class {
        "SLEEPING" | "RESTING" => "LOW",
        "ACTIVE" => "NORMAL",
        "RESTLESS" => "MEDIUM",
        "FALL_RISK" => "HIGH",
        "FALL_DETECTED" => "CRITICAL",
        _ => "UNKNOWN",
    }
}

/// Get risk level color for dashboard
pub fn get_risk_color(activity_class: &str) -> &'static str {
    match activity_class {
        "SLEEPING" | "RESTING" => "#22c55e",  // Green
        "ACTIVE" => "#3b82f6",                 // Blue
        "RESTLESS" => "#f59e0b",               // Orange
        "FALL_RISK" => "#ef4444",              // Red
        "FALL_DETECTED" => "#dc2626",          // Dark Red
        _ => "#6b7280",                        // Gray
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_activity_class() {
        assert_eq!(format_activity_class("FALL_DETECTED"), "Fall Detected");
        assert_eq!(format_activity_class("SLEEPING"), "Sleeping");
    }
    
    #[test]
    fn test_get_risk_level() {
        assert_eq!(get_risk_level("SLEEPING"), "LOW");
        assert_eq!(get_risk_level("FALL_DETECTED"), "CRITICAL");
    }
    
    #[test]
    fn test_sensor_to_features() {
        let timestamp = chrono::Utc::now();
        let features = sensor_reading_to_features(23.5, 50, 100, &timestamp, 5.0);
        
        assert_eq!(features.temperature, 23.5);
        assert_eq!(features.motion_level, 50);
        assert_eq!(features.sound_level, 100);
        assert_eq!(features.motion_trend, 5.0);
    }
}

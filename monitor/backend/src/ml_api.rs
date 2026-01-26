// ============================================================================
// ML API Integration for Rust Backend
// ============================================================================
// Add these endpoints to your api.rs file to integrate ML predictions
// ============================================================================

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use chrono::Timelike;

// Import the ML client
use crate::ml_client::{MLClient, MLFeatures, MLPrediction};

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ClassificationResponse {
    pub activity_class: String,
    pub activity_display: String,
    pub confidence: f32,
    pub risk_level: String,
    pub risk_color: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct ClassifyRequest {
    pub temperature: f32,
    pub motion_level: i32,
    pub sound_level: i32,
    #[serde(default)]
    pub hour_of_day: Option<i32>,
    #[serde(default)]
    pub motion_trend: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct MLHealthResponse {
    pub ml_service_available: bool,
    pub model_loaded: bool,
    pub classes: Vec<String>,
}

// ============================================================================
// API HANDLERS
// ============================================================================

/// GET /api/ml/health - Check ML service health
pub async fn ml_health_handler(
    ml_client: web::Data<MLClient>,
) -> HttpResponse {
    match ml_client.health_check().await {
        Ok(health) => {
            HttpResponse::Ok().json(MLHealthResponse {
                ml_service_available: true,
                model_loaded: health.model_loaded,
                classes: vec![
                    "SLEEPING".to_string(),
                    "RESTING".to_string(),
                    "ACTIVE".to_string(),
                    "RESTLESS".to_string(),
                    "FALL_RISK".to_string(),
                    "FALL_DETECTED".to_string(),
                ],
            })
        }
        Err(e) => {
            error!("ML service health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "ml_service_available": false,
                "error": e.to_string()
            }))
        }
    }
}

/// POST /api/ml/classify - Classify a sensor reading
pub async fn classify_handler(
    ml_client: web::Data<MLClient>,
    body: web::Json<ClassifyRequest>,
) -> HttpResponse {
    // Prepare features
    let hour = body.hour_of_day.unwrap_or_else(|| {
        chrono::Utc::now().hour() as i32
    });
    
    let is_night = if hour >= 22 || hour < 6 { 1 } else { 0 };
    
    let features = MLFeatures {
        temperature: body.temperature,
        motion_level: body.motion_level,
        sound_level: body.sound_level,
        hour_of_day: hour,
        is_night,
        motion_trend: body.motion_trend.unwrap_or(0.0),
    };
    
    // Call ML service
    match ml_client.classify(&features).await {
        Ok(prediction) => {
            let response = ClassificationResponse {
                activity_class: prediction.activity_class.clone(),
                activity_display: crate::ml_client::format_activity_class(&prediction.activity_class),
                confidence: prediction.confidence,
                risk_level: crate::ml_client::get_risk_level(&prediction.activity_class).to_string(),
                risk_color: crate::ml_client::get_risk_color(&prediction.activity_class).to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            
            info!("Classification: {} (confidence: {:.1}%)", 
                  response.activity_display, response.confidence * 100.0);
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("ML classification failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "ML service unavailable",
                "details": e.to_string()
            }))
        }
    }
}

/// GET /api/ml/model/info - Get model information
pub async fn model_info_handler(
    ml_client: web::Data<MLClient>,
) -> HttpResponse {
    match ml_client.get_model_info().await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            error!("Failed to get model info: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

/// Configure ML routes - add this to your main.rs App configuration
pub fn configure_ml_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ml")
            .route("/health", web::get().to(ml_health_handler))
            .route("/classify", web::post().to(classify_handler))
            .route("/model/info", web::get().to(model_info_handler))
    );
}

// ============================================================================
// INTEGRATION WITH SENSOR READINGS
// ============================================================================

/// Automatically classify sensor readings as they come in
/// Call this function whenever a new sensor reading is received
pub async fn classify_sensor_reading(
    ml_client: &MLClient,
    temperature: f32,
    motion_level: i32,
    sound_level: i32,
    previous_motion: Option<i32>,
) -> Option<MLPrediction> {
    // Calculate motion trend
    let motion_trend = match previous_motion {
        Some(prev) => (motion_level - prev) as f32,
        None => 0.0,
    };
    
    let now = chrono::Utc::now();
    let hour = now.hour() as i32;
    let is_night = if hour >= 22 || hour < 6 { 1 } else { 0 };
    
    let features = MLFeatures {
        temperature,
        motion_level,
        sound_level,
        hour_of_day: hour,
        is_night,
        motion_trend,
    };
    
    match ml_client.classify(&features).await {
        Ok(prediction) => Some(prediction),
        Err(e) => {
            error!("Failed to classify reading: {}", e);
            None
        }
    }
}

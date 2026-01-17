
use actix_web::{web, HttpRequest, HttpResponse, Error, http::header};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;
use tracing::{info, error, warn};
use std::env;
use once_cell::sync::Lazy;

use crate::db::Database;


/// Healthcare provider roles (cadres)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Cadre {
    Physician,
    Nurse,
    Physiotherapist,
    Caretaker,
}

impl std::fmt::Display for Cadre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cadre::Physician => write!(f, "physician"),
            Cadre::Nurse => write!(f, "nurse"),
            Cadre::Physiotherapist => write!(f, "physiotherapist"),
            Cadre::Caretaker => write!(f, "caretaker"),
        }
    }
}

impl std::str::FromStr for Cadre {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physician" => Ok(Cadre::Physician),
            "nurse" => Ok(Cadre::Nurse),
            "physiotherapist" => Ok(Cadre::Physiotherapist),
            "caretaker" => Ok(Cadre::Caretaker),
            _ => Err(format!("Unknown cadre: {}", s)),
        }
    }
}

/// User stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub fullname: String,
    pub email: String,
    pub cadre: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<Utc>,
}

/// User info returned to frontend (without password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub fullname: String,
    pub email: String,
    pub cadre: String,
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user_id)
    pub username: String,
    pub cadre: String,
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Signup request
#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub fullname: String,
    pub username: String,
    pub email: String,
    pub cadre: String,
    pub password: String,
}

/// Auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// =============================================================================
// JWT CONFIGURATION - USING ENVIRONMENT VARIABLES (SECURE!)
// =============================================================================

/// JWT Secret loaded from environment variable - NEVER HARDCODED!
static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        warn!("JWT_SECRET not set in environment, using default (NOT SECURE FOR PRODUCTION!)");
        // Fallback for development only - in production, this should fail
        "development_only_secret_replace_in_production".to_string()
    })
});

/// Token expiration loaded from environment variable
static TOKEN_EXPIRATION_HOURS: Lazy<i64> = Lazy::new(|| {
    env::var("TOKEN_EXPIRATION_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24)
});

/// Get JWT secret as bytes
fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.as_bytes()
}

/// Generate JWT token for a user
pub fn generate_token(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + Duration::hours(*TOKEN_EXPIRATION_HOURS);
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        cadre: user.cadre.clone(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret()),
    )
}

/// Verify and decode JWT token
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret()),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}

// =============================================================================
// PASSWORD HASHING (ENCRYPTION)
// =============================================================================

/// Hash a password using bcrypt
/// 
/// Bcrypt automatically:
/// - Generates a random salt
/// - Applies key stretching (multiple rounds)
/// - Produces a secure hash
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    // DEFAULT_COST is 12 rounds - good balance of security and speed
    hash(password, DEFAULT_COST)
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

// =============================================================================
// DATABASE OPERATIONS
// =============================================================================

/// Create users table if it doesn't exist
pub async fn init_auth_tables(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let client = db.get_client().await?;
    
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(50) UNIQUE NOT NULL,
            fullname VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            cadre VARCHAR(50) NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;
    
    info!("Users table initialized");
    Ok(())
}

/// Create a new user
pub async fn create_user(
    db: &Database,
    signup: &SignupRequest,
) -> Result<User, String> {
    // Validate cadre
    let _cadre: Cadre = signup.cadre.parse()
        .map_err(|e: String| e)?;
    
    // Hash password
    let password_hash = hash_password(&signup.password)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    // Insert user
    let client = db.get_client().await
        .map_err(|e| format!("Database connection error: {}", e))?;
    
    let row = client.query_one(
        "INSERT INTO users (username, fullname, email, cadre, password_hash)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, username, fullname, email, cadre, password_hash, created_at",
        &[&signup.username, &signup.fullname, &signup.email, &signup.cadre, &password_hash],
    ).await.map_err(|e| {
        if e.to_string().contains("unique") {
            if e.to_string().contains("username") {
                "Username already exists".to_string()
            } else if e.to_string().contains("email") {
                "Email already exists".to_string()
            } else {
                "User already exists".to_string()
            }
        } else {
            format!("Database error: {}", e)
        }
    })?;
    
    Ok(User {
        id: row.get(0),
        username: row.get(1),
        fullname: row.get(2),
        email: row.get(3),
        cadre: row.get(4),
        password_hash: row.get(5),
        created_at: row.get(6),
    })
}

/// Find user by username
pub async fn find_user_by_username(
    db: &Database,
    username: &str,
) -> Result<Option<User>, Box<dyn std::error::Error>> {
    let client = db.get_client().await?;
    
    let row = client.query_opt(
        "SELECT id, username, fullname, email, cadre, password_hash, created_at
         FROM users WHERE username = $1",
        &[&username],
    ).await?;
    
    Ok(row.map(|r| User {
        id: r.get(0),
        username: r.get(1),
        fullname: r.get(2),
        email: r.get(3),
        cadre: r.get(4),
        password_hash: r.get(5),
        created_at: r.get(6),
    }))
}

// =============================================================================
// API HANDLERS
// =============================================================================

/// POST /api/auth/signup - Register a new user
pub async fn signup_handler(
    db: web::Data<Database>,
    body: web::Json<SignupRequest>,
) -> HttpResponse {
    info!("Signup attempt for username: {}", body.username);
    
    // Validate input
    if body.username.len() < 3 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Username must be at least 3 characters".to_string(),
        });
    }
    
    if body.password.len() < 8 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Password must be at least 8 characters".to_string(),
        });
    }
    
    // Validate password strength
    let has_uppercase = body.password.chars().any(|c| c.is_uppercase());
    let has_number = body.password.chars().any(|c| c.is_numeric());
    
    if !has_uppercase || !has_number {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Password must contain at least one uppercase letter and one number".to_string(),
        });
    }
    
    match create_user(&db, &body).await {
        Ok(user) => {
            info!("User created successfully: {} ({})", user.username, user.cadre);
            
            HttpResponse::Created().json(serde_json::json!({
                "message": "Account created successfully",
                "user": UserInfo {
                    id: user.id,
                    username: user.username,
                    fullname: user.fullname,
                    email: user.email,
                    cadre: user.cadre,
                }
            }))
        }
        Err(e) => {
            warn!("Signup failed: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse { error: e })
        }
    }
}

/// POST /api/auth/login - Authenticate user
pub async fn login_handler(
    db: web::Data<Database>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    info!("Login attempt for username: {}", body.username);
    
    // Find user
    let user = match find_user_by_username(&db, &body.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: user not found: {}", body.username);
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            });
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal server error".to_string(),
            });
        }
    };
    
    // Verify password
    match verify_password(&body.password, &user.password_hash) {
        Ok(true) => {
            // Generate token
            match generate_token(&user) {
                Ok(token) => {
                    info!("Login successful for user: {} ({})", user.username, user.cadre);
                    
                    HttpResponse::Ok().json(AuthResponse {
                        token,
                        user: UserInfo {
                            id: user.id,
                            username: user.username,
                            fullname: user.fullname,
                            email: user.email,
                            cadre: user.cadre,
                        },
                    })
                }
                Err(e) => {
                    error!("Failed to generate token: {}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to generate authentication token".to_string(),
                    })
                }
            }
        }
        Ok(false) => {
            warn!("Login failed: invalid password for user: {}", body.username);
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            })
        }
        Err(e) => {
            error!("Password verification error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal server error".to_string(),
            })
        }
    }
}

/// GET /api/auth/verify - Verify JWT token
pub async fn verify_handler(req: HttpRequest) -> HttpResponse {
    // Extract token from Authorization header
    let auth_header = req.headers().get(header::AUTHORIZATION);
    
    let token = match auth_header {
        Some(value) => {
            let value_str = value.to_str().unwrap_or("");
            if value_str.starts_with("Bearer ") {
                &value_str[7..]
            } else {
                return HttpResponse::Unauthorized().json(ErrorResponse {
                    error: "Invalid authorization header format".to_string(),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Missing authorization header".to_string(),
            });
        }
    };
    
    // Verify token
    match verify_token(token) {
        Ok(claims) => {
            HttpResponse::Ok().json(serde_json::json!({
                "valid": true,
                "username": claims.username,
                "cadre": claims.cadre,
            }))
        }
        Err(e) => {
            warn!("Token verification failed: {}", e);
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid or expired token".to_string(),
            })
        }
    }
}

/// GET /api/auth/me - Get current user info
pub async fn me_handler(
    req: HttpRequest,
    db: web::Data<Database>,
) -> HttpResponse {
    // Extract and verify token
    let auth_header = req.headers().get(header::AUTHORIZATION);
    
    let token = match auth_header {
        Some(value) => {
            let value_str = value.to_str().unwrap_or("");
            if value_str.starts_with("Bearer ") {
                &value_str[7..]
            } else {
                return HttpResponse::Unauthorized().json(ErrorResponse {
                    error: "Invalid authorization header".to_string(),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Missing authorization header".to_string(),
            });
        }
    };
    
    let claims = match verify_token(token) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid or expired token".to_string(),
            });
        }
    };
    
    // Get fresh user data from database
    match find_user_by_username(&db, &claims.username).await {
        Ok(Some(user)) => {
            HttpResponse::Ok().json(UserInfo {
                id: user.id,
                username: user.username,
                fullname: user.fullname,
                email: user.email,
                cadre: user.cadre,
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: "User not found".to_string(),
            })
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Internal server error".to_string(),
            })
        }
    }
}

// =============================================================================
// MIDDLEWARE HELPER
// =============================================================================

/// Extract user from request (for protected routes)
pub fn extract_user_from_request(req: &HttpRequest) -> Option<Claims> {
    let auth_header = req.headers().get(header::AUTHORIZATION)?;
    let value_str = auth_header.to_str().ok()?;
    
    if !value_str.starts_with("Bearer ") {
        return None;
    }
    
    let token = &value_str[7..];
    verify_token(token).ok()
}

// =============================================================================
// ROUTE CONFIGURATION
// =============================================================================

/// Configure auth routes
pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/signup", web::post().to(signup_handler))
            .route("/login", web::post().to(login_handler))
            .route("/verify", web::get().to(verify_handler))
            .route("/me", web::get().to(me_handler))
    );
}

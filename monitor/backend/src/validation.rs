//! Input Validation and Sanitization Module
//!
//! Provides security functions to prevent:
//! - SQL Injection
//! - XSS (Cross-Site Scripting)
//! - Invalid FHIR data
//! - Malicious input

use once_cell::sync::Lazy;
use regex::Regex;

// ============================================================================
// REGEX PATTERNS FOR VALIDATION
// ============================================================================

/// Pattern for valid usernames (alphanumeric, underscore, hyphen, 3-30 chars)
static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]{3,30}$").unwrap());

/// Pattern for valid email addresses
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

/// Pattern to detect potential SQL injection attempts
static SQL_INJECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute|script|javascript|<script)").unwrap()
});

/// Pattern to detect XSS attempts
static XSS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(<script|javascript:|onerror=|onload=|onclick=|<iframe|<object|<embed)")
        .unwrap()
});

// ============================================================================
// VALIDATION ERRORS
// ============================================================================

#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidUsername(String),
    InvalidEmail(String),
    InvalidInput(String),
    PotentialSQLInjection(String),
    PotentialXSS(String),
    InvalidRange(String),
    InvalidFHIR(String),
    TooLong(String),
    TooShort(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidUsername(msg) => write!(f, "Invalid username: {}", msg),
            ValidationError::InvalidEmail(msg) => write!(f, "Invalid email: {}", msg),
            ValidationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ValidationError::PotentialSQLInjection(msg) => {
                write!(f, "Potential SQL injection detected: {}", msg)
            }
            ValidationError::PotentialXSS(msg) => write!(f, "Potential XSS detected: {}", msg),
            ValidationError::InvalidRange(msg) => write!(f, "Value out of range: {}", msg),
            ValidationError::InvalidFHIR(msg) => write!(f, "Invalid FHIR data: {}", msg),
            ValidationError::TooLong(msg) => write!(f, "Input too long: {}", msg),
            ValidationError::TooShort(msg) => write!(f, "Input too short: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

// ============================================================================
// INPUT SANITIZATION FUNCTIONS
// ============================================================================

/// Sanitize string input by removing potential XSS vectors
#[allow(dead_code)]
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.@,".contains(*c))
        .collect()
}

/// Sanitize HTML by escaping special characters
pub fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// Remove all HTML tags from input
#[allow(dead_code)]
pub fn strip_html_tags(input: &str) -> String {
    let tag_regex = Regex::new(r"<[^>]*>").unwrap();
    tag_regex.replace_all(input, "").to_string()
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Validate username format
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.is_empty() {
        return Err(ValidationError::InvalidUsername(
            "Username cannot be empty".to_string(),
        ));
    }

    if username.len() < 3 {
        return Err(ValidationError::TooShort(
            "Username must be at least 3 characters".to_string(),
        ));
    }

    if username.len() > 30 {
        return Err(ValidationError::TooLong(
            "Username must be at most 30 characters".to_string(),
        ));
    }

    if !USERNAME_REGEX.is_match(username) {
        return Err(ValidationError::InvalidUsername(
            "Username can only contain letters, numbers, underscore, and hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::InvalidEmail(
            "Email cannot be empty".to_string(),
        ));
    }

    if email.len() > 254 {
        return Err(ValidationError::TooLong("Email is too long".to_string()));
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::InvalidEmail(
            "Invalid email format".to_string(),
        ));
    }

    Ok(())
}

/// Check for potential SQL injection patterns
pub fn check_sql_injection(input: &str) -> Result<(), ValidationError> {
    if SQL_INJECTION_REGEX.is_match(input) {
        return Err(ValidationError::PotentialSQLInjection(
            "Input contains suspicious SQL keywords".to_string(),
        ));
    }
    Ok(())
}

/// Check for potential XSS patterns
pub fn check_xss(input: &str) -> Result<(), ValidationError> {
    if XSS_REGEX.is_match(input) {
        return Err(ValidationError::PotentialXSS(
            "Input contains suspicious script patterns".to_string(),
        ));
    }
    Ok(())
}

/// Validate string length
pub fn validate_length(
    input: &str,
    min: usize,
    max: usize,
    field_name: &str,
) -> Result<(), ValidationError> {
    if input.len() < min {
        return Err(ValidationError::TooShort(format!(
            "{} must be at least {} characters",
            field_name, min
        )));
    }
    if input.len() > max {
        return Err(ValidationError::TooLong(format!(
            "{} must be at most {} characters",
            field_name, max
        )));
    }
    Ok(())
}

/// Comprehensive input validation (username, email, etc.)
pub fn validate_text_input(input: &str, _field_name: &str) -> Result<String, ValidationError> {
    // Check for SQL injection
    check_sql_injection(input)?;

    // Check for XSS
    check_xss(input)?;

    // Sanitize and return
    Ok(sanitize_html(input))
}

// ============================================================================
// SENSOR DATA VALIDATION
// ============================================================================

/// Validate temperature reading (realistic range for room temperature)
#[allow(dead_code)]
pub fn validate_temperature(temp: f32) -> Result<(), ValidationError> {
    if !temp.is_finite() {
        return Err(ValidationError::InvalidInput(
            "Temperature must be a valid number".to_string(),
        ));
    }

    if !(0.0..=50.0).contains(&temp) {
        return Err(ValidationError::InvalidRange(format!(
            "Temperature {} is outside valid range (0-50°C)",
            temp
        )));
    }

    Ok(())
}

/// Validate motion level (0-100 percentage)
#[allow(dead_code)]
pub fn validate_motion_level(level: i32) -> Result<(), ValidationError> {
    if !(0..=100).contains(&level) {
        return Err(ValidationError::InvalidRange(format!(
            "Motion level {} is outside valid range (0-100)",
            level
        )));
    }

    Ok(())
}

/// Validate sound level (0-1023 for typical ADC)
#[allow(dead_code)]
pub fn validate_sound_level(level: i32) -> Result<(), ValidationError> {
    if !(0..=1023).contains(&level) {
        return Err(ValidationError::InvalidRange(format!(
            "Sound level {} is outside valid range (0-1023)",
            level
        )));
    }

    Ok(())
}

// ============================================================================
// FHIR VALIDATION
// ============================================================================

/// Validate FHIR resource type
#[allow(dead_code)]
pub fn validate_fhir_resource_type(resource_type: &str) -> Result<(), ValidationError> {
    const VALID_TYPES: &[&str] = &[
        "Observation",
        "Patient",
        "Bundle",
        "Practitioner",
        "Organization",
    ];

    if !VALID_TYPES.contains(&resource_type) {
        return Err(ValidationError::InvalidFHIR(format!(
            "Invalid FHIR resource type: {}",
            resource_type
        )));
    }

    Ok(())
}

/// Validate FHIR coding system URL
#[allow(dead_code)]
pub fn validate_fhir_coding_system(system: &str) -> Result<(), ValidationError> {
    const VALID_SYSTEMS: &[&str] = &[
        "http://loinc.org",
        "http://snomed.info/sct",
        "http://terminology.hl7.org/CodeSystem/observation-category",
        "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
        "http://unitsofmeasure.org",
    ];

    if !VALID_SYSTEMS.contains(&system) {
        return Err(ValidationError::InvalidFHIR(format!(
            "Invalid or unsupported FHIR coding system: {}",
            system
        )));
    }

    Ok(())
}

/// Validate observation status
#[allow(dead_code)]
pub fn validate_observation_status(status: &str) -> Result<(), ValidationError> {
    const VALID_STATUSES: &[&str] = &[
        "registered",
        "preliminary",
        "final",
        "amended",
        "corrected",
        "cancelled",
        "entered-in-error",
        "unknown",
    ];

    if !VALID_STATUSES.contains(&status) {
        return Err(ValidationError::InvalidFHIR(format!(
            "Invalid observation status: {}",
            status
        )));
    }

    Ok(())
}

// ============================================================================
// QUERY PARAMETER VALIDATION
// ============================================================================

/// Validate pagination limit
pub fn validate_limit(limit: usize, max_limit: usize) -> Result<usize, ValidationError> {
    if limit == 0 {
        return Err(ValidationError::InvalidRange(
            "Limit must be greater than 0".to_string(),
        ));
    }

    if limit > max_limit {
        return Err(ValidationError::InvalidRange(format!(
            "Limit {} exceeds maximum of {}",
            limit, max_limit
        )));
    }

    Ok(limit)
}

/// Validate time range in minutes
pub fn validate_time_range_minutes(minutes: i64) -> Result<(), ValidationError> {
    if minutes <= 0 {
        return Err(ValidationError::InvalidRange(
            "Time range must be positive".to_string(),
        ));
    }

    if minutes > 10080 {
        // 7 days
        return Err(ValidationError::InvalidRange(
            "Time range cannot exceed 7 days (10080 minutes)".to_string(),
        ));
    }

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username("a".repeat(31).as_str()).is_err()); // too long
        assert!(validate_username("user@name").is_err()); // invalid char
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user+tag@domain.co.uk").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_sql_injection_detection() {
        assert!(check_sql_injection("normal text").is_ok());
        assert!(check_sql_injection("SELECT * FROM users").is_err());
        assert!(check_sql_injection("'; DROP TABLE users--").is_err());
        assert!(check_sql_injection("1 OR 1=1").is_ok()); // basic patterns are ok
    }

    #[test]
    fn test_xss_detection() {
        assert!(check_xss("normal text").is_ok());
        assert!(check_xss("<script>alert('xss')</script>").is_err());
        assert!(check_xss("javascript:alert(1)").is_err());
        assert!(check_xss("<img onerror='alert(1)'>").is_err());
    }

    #[test]
    fn test_sanitize_html() {
        assert_eq!(sanitize_html("<script>"), "&lt;script&gt;");
        assert_eq!(sanitize_html("A & B"), "A &amp; B");
        assert_eq!(sanitize_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_validate_temperature() {
        assert!(validate_temperature(23.5).is_ok());
        assert!(validate_temperature(0.0).is_ok());
        assert!(validate_temperature(50.0).is_ok());
        assert!(validate_temperature(-10.0).is_err());
        assert!(validate_temperature(100.0).is_err());
        assert!(validate_temperature(f32::NAN).is_err());
    }

    #[test]
    fn test_validate_motion_level() {
        assert!(validate_motion_level(50).is_ok());
        assert!(validate_motion_level(0).is_ok());
        assert!(validate_motion_level(100).is_ok());
        assert!(validate_motion_level(-1).is_err());
        assert!(validate_motion_level(101).is_err());
    }

    #[test]
    fn test_validate_fhir_resource_type() {
        assert!(validate_fhir_resource_type("Observation").is_ok());
        assert!(validate_fhir_resource_type("Bundle").is_ok());
        assert!(validate_fhir_resource_type("InvalidType").is_err());
    }

    #[test]
    fn test_validate_observation_status() {
        assert!(validate_observation_status("final").is_ok());
        assert!(validate_observation_status("preliminary").is_ok());
        assert!(validate_observation_status("invalid").is_err());
    }
}

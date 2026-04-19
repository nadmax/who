use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const JWT_SECRET: &str = "changeme";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    user_id: String,
    role: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/jwt/login", post(login))
        .route("/jwt/me", get(me))
}

async fn login(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let expiry = get_current_timestamp() as usize + 3600;

    let claims = Claims {
        sub: payload.user_id,
        role: payload.role,
        exp: expiry,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to generate token: {}", e) })),
        )
    })?;

    Ok(Json(json!({ "token": token })))
}

async fn me(headers: HeaderMap) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = extract_bearer_token(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Missing or invalid Authorization header" })),
        )
    })?;

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": format!("Invalid token: {}", e) })),
        )
    })?;

    Ok(Json(json!({
        "user_id": token_data.claims.sub,
        "role": token_data.claims.role,
    })))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    auth_header.strip_prefix("Bearer ").map(|t| t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

    fn generate_token(user_id: &str, role: &str, exp: usize) -> String {
        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            exp,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_token_generation() {
        let exp = jsonwebtoken::get_current_timestamp() as usize + 3600;
        let token = generate_token("123", "admin", exp);
        assert!(!token.is_empty());
    }

    #[test]
    fn test_token_decode_valid() {
        let exp = jsonwebtoken::get_current_timestamp() as usize + 3600;
        let token = generate_token("123", "admin", exp);

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::new(Algorithm::HS256),
        );

        assert!(result.is_ok());
        let claims = result.unwrap().claims;
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_token_decode_invalid_signature() {
        let exp = jsonwebtoken::get_current_timestamp() as usize + 3600;
        let token = generate_token("123", "admin", exp);

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(b"wrongsecret"),
            &Validation::new(Algorithm::HS256),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_token_decode_expired() {
        let exp = jsonwebtoken::get_current_timestamp() as usize - 3600; // 1 hour in the past
        let token = generate_token("123", "admin", exp);

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::new(Algorithm::HS256),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_valid() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer mytoken123".parse().unwrap(),
        );
        assert_eq!(
            extract_bearer_token(&headers),
            Some("mytoken123".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let headers = axum::http::HeaderMap::new();
        assert_eq!(extract_bearer_token(&headers), None);
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Basic mytoken123".parse().unwrap(),
        );
        assert_eq!(extract_bearer_token(&headers), None);
    }
}

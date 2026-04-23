use axum::http::HeaderMap;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const JWT_SECRET: &str = "changeme";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn create_token(user_id: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = get_current_timestamp() as usize + 3600;
    let claims = Claims { sub: user_id.to_string(), role: role.to_string(), exp };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
}

pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}

pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    auth_header.strip_prefix("Bearer ").map(|t| t.to_string())
}

pub fn claims_to_json(claims: &Claims) -> Value {
    json!({ "user_id": claims.sub, "role": claims.role })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_decode_token() {
        let token = create_token("123", "admin").unwrap();
        assert!(!token.is_empty());

        let claims = decode_token(&token).unwrap();
        assert_eq!(claims.sub, "123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_decode_invalid_signature() {
        let token = create_token("123", "admin").unwrap();
        let result = jsonwebtoken::decode::<Claims>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(b"wrongsecret"),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_expired_token() {
        let claims = Claims {
            sub: "123".to_string(),
            role: "admin".to_string(),
            exp: jsonwebtoken::get_current_timestamp() as usize - 3600,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(b"changeme"),
        ).unwrap();
        assert!(decode_token(&token).is_err());
    }

    #[test]
    fn test_extract_bearer_token_valid() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(axum::http::header::AUTHORIZATION, "Bearer mytoken123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), Some("mytoken123".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        assert_eq!(extract_bearer_token(&axum::http::HeaderMap::new()), None);
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(axum::http::header::AUTHORIZATION, "Basic mytoken123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), None);
    }
}
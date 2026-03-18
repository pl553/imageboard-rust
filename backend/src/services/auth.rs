use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::Config;
use crate::domain::{AdminClaims, AuthToken, DomainError, DomainResult};
use crate::repositories::AdminRepository;

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn login(&self, username: &str, password: &str) -> DomainResult<AuthToken>;
    fn verify(&self, token: &str) -> DomainResult<AdminClaims>;
    async fn change_password(
        &self,
        admin_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> DomainResult<()>;
}

#[derive(Clone)]
pub struct AuthServiceImpl {
    admin_repo: Arc<dyn AdminRepository>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: i64,
}

impl AuthServiceImpl {
    pub fn new(admin_repo: Arc<dyn AdminRepository>, config: &Config) -> Self {
        let secret = config.jwt.secret.as_bytes();
        Self {
            admin_repo,
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            expiry_hours: config.jwt.expiry_hours,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    admin_id: i64,
    username: String,
    exp: i64,
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn login(&self, username: &str, password: &str) -> DomainResult<AuthToken> {
        let admin = self
            .admin_repo
            .find_by_username(username)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        if !verify_password(password, &admin.password_hash)? {
            return Err(DomainError::InvalidCredentials);
        }

        let expires_at = Utc::now() + Duration::hours(self.expiry_hours);
        let claims = JwtClaims {
            sub: "admin".to_string(),
            admin_id: admin.id,
            username: admin.username,
            exp: expires_at.timestamp(),
        };

        let token = jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| DomainError::Internal(format!("jwt encode error: {}", e)))?;

        Ok(AuthToken { token, expires_at })
    }

    fn verify(&self, token: &str) -> DomainResult<AdminClaims> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let data = jsonwebtoken::decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => DomainError::TokenExpired,
                _ => DomainError::InvalidToken,
            })?;

        Ok(AdminClaims {
            admin_id: data.claims.admin_id,
            username: data.claims.username,
            exp: data.claims.exp,
        })
    }

    async fn change_password(
        &self,
        admin_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> DomainResult<()> {
        if new_password.len() < 8 {
            return Err(DomainError::Validation(
                "new_password must be at least 8 characters".to_string(),
            ));
        }

        let admin = self
            .admin_repo
            .find_by_id(admin_id)
            .await?
            .ok_or(DomainError::Unauthorized)?;

        if !verify_password(old_password, &admin.password_hash)? {
            return Err(DomainError::Unauthorized);
        }

        if verify_password(new_password, &admin.password_hash)? {
            return Err(DomainError::Validation(
                "new_password must be different from old password".to_string(),
            ));
        }

        let hash = hash_password(new_password)?;
        self.admin_repo.change_password(admin_id, &hash).await?;
        Ok(())
    }
}

pub fn hash_password(password: &str) -> DomainResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| DomainError::Internal(format!("bcrypt hash error: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> DomainResult<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e| DomainError::Internal(format!("bcrypt verify error: {}", e)))
}


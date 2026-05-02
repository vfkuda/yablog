use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use thiserror::Error;

use crate::{
    data::user_repository::UserRepository,
    domain::{
        error::DomainError,
        user::{LoginRequest, RegistrationRequest, User},
    },
    infrastructure::jwt::JwtService,
};

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("failed to hash password")]
    PasswordHash,
    #[error("token generation failed")]
    TokenGeneration,
}

#[derive(Clone)]
pub struct AuthService {
    user_repository: Arc<UserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(user_repository: Arc<UserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repository,
            jwt_service,
        }
    }

    pub async fn register(
        &self,
        request: RegistrationRequest,
    ) -> Result<(String, User), AuthServiceError> {
        let password_hash = hash_password(&request.password)?;
        let user = self
            .user_repository
            .create_user(&request.username, &request.email, &password_hash)
            .await?;
        let token = self
            .jwt_service
            .generate_token(user.id, &user.username)
            .map_err(|_| AuthServiceError::TokenGeneration)?;
        Ok((token, user))
    }

    pub async fn login(&self, request: LoginRequest) -> Result<(String, User), AuthServiceError> {
        let user = self
            .user_repository
            .find_by_username(&request.username)
            .await?;

        verify_password(&request.password, &user.password_hash)?;
        let token = self
            .jwt_service
            .generate_token(user.id, &user.username)
            .map_err(|_| AuthServiceError::TokenGeneration)?;

        Ok((token, user))
    }
}

fn hash_password(password: &str) -> Result<String, AuthServiceError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hashed| hashed.to_string())
        .map_err(|_| AuthServiceError::PasswordHash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), AuthServiceError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AuthServiceError::Domain(DomainError::InvalidCredentials))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthServiceError::Domain(DomainError::InvalidCredentials))
}

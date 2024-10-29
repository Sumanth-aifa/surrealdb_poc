use axum::{
    extract::Request, http::header::AUTHORIZATION, http::StatusCode, middleware::Next,
    response::Response, Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::todo::Db;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Option<Thing>,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginCreds {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    message: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn register_user(
    db: Db,
    Json(user): Json<User>,
) -> Result<Json<RegisterResponse>, (StatusCode, String)> {
    let existing_user: Option<User> = db
        .select(("user", &user.email))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing_user.is_some() {
        return Err((StatusCode::CONFLICT, "User already exists".to_string()));
    }

    let hashed_pass = hash(user.password, DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let user_val = User {
        id: None,
        email: user.email.clone(),
        password: hashed_pass,
    };
    let _created_user: Option<User> = db
        .create(("user", &user.email))
        .content(user_val)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RegisterResponse {
        message: "User created Successfully".to_string(),
        email: user.email.clone(),
    }))
}

pub async fn login_user(
    db: Db,
    Json(user): Json<LoginCreds>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user_cred: Option<User> = db
        .select(("user", &user.email))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let user_main =
        user_cred.ok_or((StatusCode::UNAUTHORIZED, "Invalid Credentials ".to_string()))?;
    let password_matches = verify(user.password.as_bytes(), &user_main.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !password_matches {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password".to_string()));
    }

    let expiration = chrono::Utc::now().timestamp() as usize + 1 * 3600;

    let claims = Claims {
        sub: user.email.clone(),
        exp: expiration,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"hello"),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthResponse { token }))
}

async fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"hello"),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = req.headers_mut().get(AUTHORIZATION);
    let auth_header = match auth_header {
        Some(header) => header.to_str().map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                "Empty header is not allowed".to_string(),
            )
        })?,
        None => {
            return Err({
                (
                    StatusCode::FORBIDDEN,
                    "Please add the JWT token to the header".to_string(),
                )
            })
        }
    };

    let mut header = auth_header.split_whitespace();
    let (_, token) = (header.next(), header.next());
    let token = token.unwrap_or("token not found ");
    let claims = verify_token(token)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

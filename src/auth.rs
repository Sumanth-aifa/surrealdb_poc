use axum::{
    extract::Request, http::header::AUTHORIZATION, http::StatusCode, middleware::Next,
    response::Response, Json,
};

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::opt::auth::Root;

use crate::todo::Db;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginCreds {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in_hours: i64,
}

pub async fn auth_middleware(
    db: Db,
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

    match db.authenticate(token).await {
        Ok(_) => Ok(next.run(req).await),
        Err(e) => {
            if e.to_string().contains("expired") {
                Err((
                    StatusCode::UNAUTHORIZED,
                    "Token has expired please login again".to_string(),
                ))
            } else {
                Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
            }
        }
    }
}

pub async fn login_user(
    db: Db,
    Json(credentials): Json<LoginCreds>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let root = Root {
        username: credentials.username.as_str(),
        password: credentials.password.as_str(),
    };

    // Sign in and get the token
    let response = db
        .signin(root)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let _ = db
        .use_ns("Rise")
        .use_db("TodoSQL")
        .await
        .map_err(|_| "Something went wrong with namespace or database");
    // Get the token as string
    let token = response.as_insecure_token();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .timestamp();
    let now = Utc::now().timestamp();
    let expires_at = (expires_at - now) / 3600;

    Ok(Json(AuthResponse {
        token: token.to_string(),
        expires_in_hours: expires_at,
    }))
}

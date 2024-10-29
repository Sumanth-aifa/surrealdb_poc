use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Id not provided")]
    IdNotFound,
    #[error("Error in database: {0}")]
    DbError(#[from] surrealdb::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let code = match self {
            Error::IdNotFound => StatusCode::BAD_REQUEST,
            Error::DbError(_) => StatusCode::BAD_REQUEST,
        };
        (code, self.to_string()).into_response()
    }
}

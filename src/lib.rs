mod error;
// mod person;
mod todo;

use axum::{
    routing::{get, post},
    Router,
};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub fn create_router(db: Surreal<Any>) -> Router {
    Router::new()
        .route("/create_todo", post(todo::create_todo))
        .route("/get_todos", get(todo::list_todos))
        .with_state(db)
}

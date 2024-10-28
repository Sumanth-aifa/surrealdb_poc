mod error;
mod todo;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub fn create_router(db: Surreal<Any>) -> Router {
    Router::new()
        .route("/create_todo", post(todo::create_todo))
        .route("/get_todos", get(todo::list_todos))
        .route("/update_todo", put(todo::update_todo))
        .route("/delete_todo", delete(todo::delete_todo))
        .with_state(db)
}

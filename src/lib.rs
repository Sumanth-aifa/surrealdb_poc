mod auth;
mod books;
mod error;
mod todo;

use auth::auth_middleware;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};

use surrealdb::engine::any::Any;

use surrealdb::Surreal;

pub type APIResult<T> = Result<Json<T>, error::Error>;
pub type APIResultStatus = Result<Json<surrealdb::Value>, error::Error>;

pub fn create_router(db: Surreal<Any>) -> Router {
    let public_routes = Router::new().route("/login", post(auth::login_user));

    let todo_routes = Router::new()
        // Todoapi
        .route("/create_todo", post(todo::create_todo))
        .route("/get_todo", get(todo::list_todo))
        .route("/update_todo", put(todo::update_todo))
        .route("/delete_todo", delete(todo::delete_todo))
        // SQL todoapi
        .route("/create_book", post(books::create_book))
        .route("/get_book", get(books::list_book))
        .route("/update_book", put(books::update_book))
        .route("/delete_book", delete(books::delete_book))
        // State with layer
        .layer(middleware::from_fn_with_state(db.clone(), auth_middleware));
    Router::new()
        .merge(public_routes)
        .merge(todo_routes)
        .with_state(db)
}

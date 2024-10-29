use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

use crate::error::Error;
use crate::APIResult;

pub type Db = State<Surreal<Any>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Todo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
    pub completed: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CreateTodoInput {
    pub todos: Vec<Todo>,
}

pub async fn create_todo(db: Db, Json(input): Json<CreateTodoInput>) -> APIResult<Vec<Todo>> {
    let mut created_todos: Vec<Todo> = Vec::new();

    for todo in input.todos {
        match db.create("todo").content(todo).await {
            Ok(Some(created)) => {
                created_todos.push(created);
            }
            Ok(None) => {
                return Ok(Default::default());
            }
            Err(e) => {
                return Err(Error::from(e));
            }
        }
    }

    Ok(Json(created_todos))
}

pub async fn list_todo(db: Db) -> APIResult<Vec<Todo>> {
    let todos: Vec<Todo> = db.select("todo").await?;
    Ok(Json(todos))
}

pub async fn update_todo(
    db: Db,
    Query(params): Query<HashMap<String, String>>,
    Json(input): Json<Todo>,
) -> APIResult<Option<Todo>> {
    let id = params.get("id").ok_or_else(|| Error::IdNotFound)?;
    let updated_todo = db.update(("todo", id)).content(input).await?;
    Ok(Json(updated_todo))
}

pub async fn delete_todo(
    db: Db,
    Query(params): Query<HashMap<String, String>>,
) -> APIResult<Option<Todo>> {
    let id = params.get("id").ok_or_else(|| Error::IdNotFound)?;
    let deleted = db.delete(("todo", id)).await?;
    Ok(Json(deleted))
}

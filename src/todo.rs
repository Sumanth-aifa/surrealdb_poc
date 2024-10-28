use crate::error::Error;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

type Db = State<Surreal<Any>>;

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

pub async fn create_todo(
    db: Db,
    Json(input): Json<CreateTodoInput>,
) -> Result<Json<Vec<Todo>>, Error> {
    println!("Received input: {:?}", input);
    let mut created_todos: Vec<Todo> = Vec::new();

    for todo in input.todos {
        match db.create("todo").content(todo).await {
            Ok(Some(created)) => {
                created_todos.push(created);
            }
            Ok(None) => {
                println!("Failed to create todo - returned None");
                return Ok(Default::default());
            }
            Err(e) => {
                println!("Error creating todo: {:?}", e);
                return Err(Error::from(e));
            }
        }
    }

    println!("Final created_todos: {:?}", created_todos);
    Ok(Json(created_todos))
}

pub async fn list_todos(db: Db) -> Result<Json<Vec<Todo>>, Error> {
    let todos: Vec<Todo> = db.select("todo").await?;

    Ok(Json(todos))
}

// pub async fn update_todo(
//     db: Db,
//     Query(id): Query<String>,
//     Json(input): Json<Todo>,
// ) -> Result<Json<Option<Todo>>, Error> {
//     let updated_todo = db.update("todo", &*id).content(input).await?;
//     Ok(Json(updated_todo))
// }

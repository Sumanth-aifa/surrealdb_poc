use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use crate::todo::Db;
use crate::{APIResult, APIResultStatus};

const QUERY_TABLE: &str = "SELECT * FROM Books";
const QUERY_INSERT: &str = r#"
BEGIN TRANSACTION;

LET $count = SELECT COUNT(id) FROM Books WHERE bookName=$bookname;

IF $count.len() == 0 {
    INSERT INTO Books $record;
    RETURN {
        success: true,
        message: "The book " + <string>$bookname + " was added"
    };
} ELSE {
    RETURN {
        success: false,
        message: "The book " + <string>$bookname + " already exists"
    };
};

COMMIT TRANSACTION;
"#;
const QUERY_UPDATE: &str = r#"
BEGIN TRANSACTION;

LET $count = SELECT COUNT(id) FROM Books WHERE bookName=$bookname;

IF $count.len() < 1 {
    RETURN {
        success: false,
        message: "The record " + <string>$id + " does not exist"
    };
} ELSE {
    UPDATE Books MERGE $record WHERE id=type::thing("Books", $id);
    RETURN {
        success: true,
        message: "The record " + <string>$id + " was updated"
    };
};

COMMIT TRANSACTION;
"#;
const QUERY_DELETE: &str = r#"
DELETE Books WHERE id=type::thing("Books", $id) RETURN {
    success: true,
    message: "the item " + <string>$id + " was deleted"
};
"#;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    id: Option<surrealdb::sql::Thing>,
    book_name: String,
    author_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Id {
    id: String,
}

pub async fn create_book(db: Db, Json(book): Json<Book>) -> APIResultStatus {
    Ok(Json(
        db.query(QUERY_INSERT)
            .bind(("record", book.clone()))
            .bind(("bookname", book.book_name.clone()))
            .await?
            .take(0)?,
    ))
}

pub async fn list_book(db: Db) -> APIResult<Vec<Book>> {
    Ok(Json(db.query(QUERY_TABLE).await?.take(0)?))
}

pub async fn update_book(
    db: Db,
    Query(Id { id }): Query<Id>,
    Json(book): Json<Book>,
) -> APIResultStatus {
    Ok(Json(
        db.query(QUERY_UPDATE)
            .bind(("record", book.clone()))
            .bind(("bookname", book.book_name.clone()))
            .bind(("id", id))
            .await?
            .take(0)?,
    ))
}

pub async fn delete_book(db: Db, Query(Id { id }): Query<Id>) -> APIResultStatus {
    Ok(Json(
        db.query(QUERY_DELETE)
            .bind(("id", id))
            .await?
            .take(0)?,
    ))
}

use std::env;
use surrealdb::engine::any;
use surrealdb::opt::Config;
use surrealdb_rust::create_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allow the endpoint to be configured via a `SURREALDB_ENDPOINT` environment variable
    // or fallback to memory. This makes it possible to configure the endpoint at runtime.
    let endpoint =
        env::var("SURREALDB_ENDPOINT").unwrap_or_else(|_| "http://localhost:8000".to_owned());

    let config = Config::new();

    // Create the database connection.
    let db = any::connect((endpoint, config)).await?;

    let listener = TcpListener::bind("localhost:8080").await?;
    let router = create_router(db);
    println!("server running at 8080");
    axum::serve(listener, router).await?;

    Ok(())
}

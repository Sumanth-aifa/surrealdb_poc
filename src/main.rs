use std::env;
use surrealdb::engine::any;
use surrealdb::opt::auth::Root;
use surrealdb::opt::Config;
use surrealdb_rust::create_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allow the endpoint to be configured via a `SURREALDB_ENDPOINT` environment variable
    // or fallback to memory. This makes it possible to configure the endpoint at runtime.
    let endpoint =
        env::var("SURREALDB_ENDPOINT").unwrap_or_else(|_| "http://localhost:8000".to_owned());

    let root = Root {
        username: "admin",
        password: "nopwd",
    };

    // Activate authentication on local engines by supplying the root user to be used.
    let config = Config::new().user(root);

    // Create the database connection.
    let db = any::connect((endpoint, config)).await?;

    // Sign in as root.
    db.signin(root).await?;

    // Configure the namespace amd database to use.
    db.use_ns("Rise").use_db("TodoSQL").await?;

    let listener = TcpListener::bind("localhost:8080").await?;
    let router = create_router(db);
    println!("server running at 8080");
    axum::serve(listener, router).await?;

    Ok(())
}

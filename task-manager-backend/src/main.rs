use std::net::SocketAddr;

use axum::middleware::from_fn;
use axum::routing::{post, delete, patch, get};
use axum::Router;
use color_eyre::Result;
use surrealdb::engine::remote::ws::Client;
use surrealdb::{Surreal, engine::remote::ws::Ws};
use dotenvy;

mod task;
mod db_types;
mod auth;
mod cors;

#[derive(Clone)]
pub struct SecretKey(String);

#[derive(Clone)]
pub struct ServerState {
    key: String,
    db: Surreal<Client>
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let secret_key = dotenvy::var("SECRET_KEY")?;
    let db = Surreal::new::<Ws>("localhost:8000").await?;
    db.use_ns("test").use_db("test").await?;

    let state = ServerState {
        key: secret_key,
        db
    };

    let app = Router::new()
        .route("/login", post(auth::login))
        .route("/task", post(task::create_task))
        .route("/task", get(task::get_tasks))
        .route("/task/:id", delete(task::delete_task))
        .route("/task/:id/title", patch(task::update_title))
        .route("/task/:id/priority", patch(task::update_priority))
        .route("/task/:id/due", patch(task::update_due_date))
        .route("/task/:id/done", patch(task::update_done))
        .layer(from_fn(auth::auth_middleware))
        .layer(from_fn(cors::cors_middleware))
        .with_state(state);

    println!("Hello, world!");

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}


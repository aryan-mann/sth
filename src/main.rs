use actix_web::{web, App, HttpServer, rt};
use sqlx::{postgres::PgPoolOptions};

use crate::task_runner::TaskRunner;

pub mod common;
pub mod db;
pub mod routes;
pub mod models;
pub mod task_runner;

// TODO: Load from environment variable
const POSTGRES_CONNECTION_STRING: &str = "postgresql://postgres:postgres@localhost:5432/svix";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: Switch to logging
    println!("\n\tSvix Take-Home\n");

    // Connecting to database
    println!("Connecting to database.");
    let db_pool = PgPoolOptions::new()
        .connect(POSTGRES_CONNECTION_STRING)
        .await
        .expect("Unable to connect to database");
    println!("Connected to database.\n");

    // Spawn a thread for the Task Runner
    let runner_db = db_pool.clone();
    rt::spawn(async {
        let mut task_runner = TaskRunner::new(runner_db);
        task_runner.start().await;
    });
    
    // Spawn API workers
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .route("/health", web::get().to(routes::health::health_check))
            // TODO: Move the service creation into routes/task
            .service(
                web::scope("/task")
                    .route("/", web::post().to(routes::task::create_task))
                    .route("/", web::get().to(routes::task::get_all_tasks))
                    .route("/{id}", web::get().to(routes::task::show_task))
                    .route("/{id}", web::delete().to(routes::task::delete_task))
            )
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
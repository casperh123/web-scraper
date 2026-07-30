use actix_web::{web, App, HttpServer};
use database::db::db::connection;
use sea_orm::DatabaseConnection;
mod websites;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let db = connection()
        .await
        .expect("Could not connect to database");

    let state = web::Data::new(AppState {
        db: db.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(websites::handler::configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

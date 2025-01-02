use std::env;
use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use dotenv::dotenv;
use sqlx::PgPool;
use jwt_auth_lib::{
    models::appstate::Appstate
};

#[tokio::main]
async fn main() {
    // load environment
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").unwrap();

    let psql_url = env::var("PSQL_URL").unwrap();
    let pool = PgPool::connect(&psql_url).await.unwrap();
    let shared_pool = Arc::new(pool);

    let appstate = Appstate {
        db_pool: shared_pool,
        jwt_secret,
    };


    // set up axum
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

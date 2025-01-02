use std::env;
use std::sync::Arc;
use axum::Router;
use axum::routing::{post};
use dotenv::dotenv;
use sqlx::PgPool;
use jwt_auth_lib::{
    models::appstate::Appstate,
    handlers::users::*,
};

#[tokio::main]
async fn main() {
    // load environment
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").unwrap();

    let psql_url = env::var("PSQL_URL").unwrap();
    let pool = PgPool::connect(&psql_url).await.unwrap();
    let shared_pool = Arc::new(pool);

    let appstate = Arc::new(Appstate {
        db_pool: shared_pool,
        jwt_secret,
    });


    // set up axum
    let app = Router::new()
        .route("/v1/user/new", post(new::new)).with_state(appstate.clone())
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

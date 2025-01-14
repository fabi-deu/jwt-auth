use axum::handler::Handler;
use axum::routing::{get, post};
use axum::{middleware, Extension, Router};
use dotenv::dotenv;
use jwt_auth_lib::handlers::users::authorize::auth;
use jwt_auth_lib::{
    handlers::users::*,
    models::appstate::Appstate,
};
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() {
    // load environment
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").unwrap();

    // postgres connection
    let psql_url = env::var("PSQL_URL").unwrap();
    let pool = PgPool::connect(&psql_url).await.unwrap();
    let shared_pool = Arc::new(pool);

    let appstate = Arc::new(Appstate {
        db_pool: shared_pool,
        jwt_secret,
    });


    // set up axum
    // ! ERROR: AUTH LAYER APPLIES TO EVERY LAYER -> CANT CREATE USER WITHOUT AUTH
    let app = Router::new()
        .route("/v1/user/new", post(new::new)).layer(CookieManagerLayer::new())
        .route("/v1/", get(hello))
        .with_state(appstate.clone())
        .layer(middleware::from_fn(auth))
        .layer(Extension(appstate.clone()))// Apply the auth middleware
        .layer(CookieManagerLayer::new()) // Apply the cookie manager layer
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}


async fn hello() -> &'static str {
    "hello world"
}
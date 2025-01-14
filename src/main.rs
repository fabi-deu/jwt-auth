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
    let app = Router::new()
        .route(
            "/v1/user/new",
            post(new::new)
                .with_state(appstate.clone())
                .layer(CookieManagerLayer::new())
        )
        // ! ERROR: something with auth layer
        .route(
            "/v1/auth",
            get(test)
                .layer(middleware::from_fn(auth))
                .layer(Extension(appstate.clone()))
                /* .with_state() does work here so im making use of extensions */
                .layer(CookieManagerLayer::new())
        )
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}


async fn test() -> &'static str {
    "Hello World!"
}
use axum::routing::{get, post, put};
use axum::{middleware, Extension, Json, Router};
use dotenv::dotenv;
use jwt_auth_lib::handlers::users::authenticate::auth;
use jwt_auth_lib::{
    handlers::users::*,
    models::appstate::Appstate,
};
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tower::ServiceBuilder;
use jwt_auth_lib::handlers::users::login::login;
use jwt_auth_lib::models::user::{AuthUser, User};

#[tokio::main]
async fn main() {
    // load environment
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").unwrap();

    // postgres connection
    let psql_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&psql_url).await.unwrap();
    let shared_pool = Arc::new(pool);

    let appstate = Arc::new(Appstate {
        db_pool: shared_pool,
        jwt_secret,
    });

    let protected_routes = Router::new()
        .route("/v1/auth_test", get(test))
        .route("/v1/password/change", put(updating::password::change::change_password))
        .route("/v1/username/change", put(updating::username::change::change_username))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(auth))
                .layer(Extension(appstate.clone()))
        );


    let public_routes = Router::new()
        .route("/v1/user/new", post(new::new))
        .route("/v1/user/login", post(login))
    ;


    // set up axum
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(Extension(appstate.clone()))
        .with_state(appstate.clone())
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

#[axum_macros::debug_handler]
async fn test(
    ext: Extension<AuthUser>
) -> Json<User> {
    Json(ext.0.0)
}
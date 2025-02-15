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
use axum::http::Method;
use axum_extra::extract::cookie::Key;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::{Any, CorsLayer};
use jwt_auth_lib::handlers::users::login::login;
use jwt_auth_lib::handlers::users::refresh::refresh_token;
use jwt_auth_lib::models::appstate::AppstateWrapper;
use jwt_auth_lib::models::user::{AuthUser, User};

#[tokio::main]
async fn main() {
    // load environment vars
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").unwrap();
    let cookie_secret = env::var("COOKIE_SECRET").unwrap();

    // postgres connection
    let psql_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(&psql_url).await.unwrap();
    let shared_pool = Arc::new(pool);

    // create appstate
    let appstate = Arc::new(Appstate::new(
        shared_pool,
        jwt_secret,
        Key::try_from(cookie_secret.as_bytes()).unwrap()
    ));
    let wrapped_appstate = AppstateWrapper(appstate);

    // set up http server
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_origin(Any);

    let protected_routes = Router::new()
        .route("/v1/auth_test", get(test))
        .route("/v1/user/password/change", put(updating::password::change::change_password))
        .route("/v1/user/username/change", put(updating::username::change::change_username))
        .route("/v1/user/refresh_token", get(refresh_token))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(auth))
                .layer(Extension(wrapped_appstate.clone()))
        );

    let public_routes = Router::new()
        .route("/v1/user/new", post(new::new))
        .route("/v1/user/login", post(login))
    ;


    // set up axum
    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(
            ServiceBuilder::new()
                .layer(Extension(wrapped_appstate.clone()))
                .layer(TraceLayer::new_for_http())
                .layer(cors)
        )
        .with_state(wrapped_appstate.clone())
    ;

    println!("Listening & serving on 0.0.0.0:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

#[axum_macros::debug_handler]
async fn test(
    ext: Extension<AuthUser>
) -> Json<User> {
    Json(ext.0.0)
}
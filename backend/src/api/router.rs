use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::api::handlers;
use crate::api::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                // boards
                .route("/boards", get(handlers::boards::list_boards).post(handlers::boards::create_board))
                .route("/boards/{slug}", get(handlers::boards::get_board).delete(handlers::boards::delete_board))
                // threads
                .route("/boards/{slug}/threads", get(handlers::threads::list_threads).post(handlers::threads::create_thread))
                .route("/boards/{slug}/threads/{thread_number}", get(handlers::threads::get_thread).delete(handlers::threads::delete_thread))
                // posts
                .route("/boards/{slug}/threads/{thread_number}/posts", post(handlers::posts::create_post))
                .route("/boards/{slug}/posts/{post_number}", delete(handlers::posts::delete_post))
                // auth
                .route("/auth/login", post(handlers::auth::login))
                .route("/auth/me", get(handlers::auth::me))
                .route("/auth/change-password", post(handlers::auth::change_password))
                // images
                .route("/images/{filename}", get(handlers::images::get_image))
                .route("/images/thumb/{filename}", get(handlers::images::get_thumbnail)),
        )
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}


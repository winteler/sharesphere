use std::env;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;

use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_session::{Key, SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use sqlx::PgPool;

use utils::utils::ssr::{create_db_pool, AuthSession};
use utils::user::ssr::UserLockCache;
use utils::user::User;

use app::{
    app::*,
    post::ssr::update_post_scores,
};

use crate::fallback::file_and_error_handler;
use crate::state::AppState;

mod fallback;
mod state;

pub const SESSION_KEY_ENV : &str = "SESSION_KEY";
pub const SESSION_DB_KEY_ENV : &str = "SESSION_DB_KEY";
pub const USER_LOCK_CACHE_SIZE_ENV : &str = "USER_LOCK_CACHE_SIZE";
pub const POST_SCORE_UPDATE_INTERVAL_S_ENV : &str = "POST_SCORE_UPDATE_INTERVAL_S";

pub fn get_session_key() -> Key {
    match env::var(SESSION_KEY_ENV) {
        Ok(key) => {
            log::debug!("Got session key from env variable.");
            Key::from(&key.into_bytes())
        },
        Err(_) => {
            log::info!("Could not find session key in env variable, generate one.");
            Key::generate()
        }
    }
}

pub fn get_session_db_key() -> Key {
    match env::var(SESSION_DB_KEY_ENV) {
        Ok(key) => {
            log::debug!("Got session db key from env variable.");
            Key::from(&key.into_bytes())
        },
        Err(_) => {
            log::info!("Could not find session db key in env variable, generate one.");
            Key::generate()
        }
    }
}

pub fn get_user_lock_cache_size() -> NonZeroUsize {
    let default_size = NonZeroUsize::new(1000000).expect("Should initialize NonZeroUsize");
    match env::var(USER_LOCK_CACHE_SIZE_ENV) {
        Ok(value) => {
            log::debug!("Got session db key from env variable.");
            match NonZeroUsize::from_str(&value) {
                Ok(value) => value,
                Err(_) => {
                    log::error!("Could not parse user lock cache size as NonZeroUsize.");
                    default_size
                }
            }
        },
        Err(_) => {
            log::debug!("Could not find user lock cache size in env variable, take default value.");
            default_size
        }
    }
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    log::debug!("{path:?}");

    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.db_pool.clone());
            provide_context(app_state.user_lock_cache.clone());
        },
        request,
    )
    .await
}

 async fn leptos_routes_handler(
     auth_session: AuthSession,
     app_state: State<AppState>,
     req: Request<AxumBody>,
 ) -> Response {
     let leptos_options = app_state.leptos_options.clone();
     let db_pool = app_state.db_pool.clone();
     let user_lock_cache = app_state.user_lock_cache.clone();
     let handler = leptos_axum::render_route_with_context(
         app_state.routes.clone(),
         move || {
             provide_context(auth_session.clone());
             provide_context(db_pool.clone());
             provide_context(user_lock_cache.clone());
         },
         move || shell(leptos_options.clone()),
     );
     handler(app_state, req).await.into_response()
 }

async fn update_post_scores_loop(db_pool: PgPool) {
    let default_interval_seconds = 5 * 60;
    let wait_interval_seconds = match env::var(POST_SCORE_UPDATE_INTERVAL_S_ENV) {
        Ok(wait_interval_string) => wait_interval_string.parse().unwrap_or(default_interval_seconds),
        _ => default_interval_seconds,
    };
    let interval = tokio::time::Duration::from_secs(wait_interval_seconds); // 5 minutes

    loop {
        let result = update_post_scores(&db_pool).await;
        if let Err(e) = result {
            log::error!("Failed to updated posts' ranking timestamps with error: {e}");
        } else {
            log::debug!("Successfully updated posts' ranking timestamps");
        }
        tokio::time::sleep(interval).await;
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("Should be able to initialize logging.");

    let subscriber = tracing_subscriber::fmt().with_max_level(tracing::Level::ERROR).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    let pool = create_db_pool().await.expect("Failed to create db pool");

    // Start a task to periodically update post scores
    tokio::spawn(update_post_scores_loop(pool.clone()));

    let session_config = SessionConfig::default()
        .with_table_name("sessions")
        // 'Key::generate()' will generate a new key each restart of the server.
        // If you want it to be more permanent then generate and set it to a config file.
        // If with_key() is used it will set all cookies as private, which guarantees integrity, and authenticity.
        .with_key(get_session_key())
        // This is how we would Set a Database Key to encrypt as store our per session keys.
        // This MUST be set in order to use SecurityMode::PerSession.
        .with_database_key(get_session_db_key());

    let auth_config = AuthConfig::<i64>::default();
    let session_store = SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config).await.unwrap();

    sqlx::migrate!("../migrations/")
        .run(&pool)
        .await
        .expect("Should be able to run SQLx migrations.");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        db_pool: pool.clone(),
        user_lock_cache: Arc::new(UserLockCache::new(get_user_lock_cache_size())),
        routes: routes.clone(),
    };

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler)
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .layer(
            AuthSessionLayer::<User, i64, SessionPgPool, PgPool>::new(
                Some(pool)
            ).with_config(auth_config)
        )
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

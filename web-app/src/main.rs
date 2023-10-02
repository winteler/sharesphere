use cfg_if::cfg_if;

// boilerplate to run in different modes
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            response::{Response, IntoResponse},
            routing::get,
            extract::{Path, State, RawQuery},
            http::{Request, header::HeaderMap},
            body::Body as AxumBody,
            Router,
        };
        use leptos::*;
        use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
        use axum_session::{SessionPgPool, SessionConfig, SessionLayer, SessionStore, Key, SecurityMode};

        use std::env;
        use sqlx::postgres::PgPoolOptions;
        use anyhow::{Context};

        use project_web_app::app::*;
        use project_web_app::auth::*;
        use project_web_app::fileserv::file_and_error_handler;
        use project_web_app::state::AppState;

        pub const DB_URL_ENV : &str = "DATABASE_URL";
        pub const SESSION_KEY_ENV : &str = "SESSION_KEY";
        pub const SESSION_DB_KEY_ENV : &str = "SESSION_DB_KEY";

        #[cfg(feature = "ssr")]
        pub fn get_session_key() -> Key {
            match env::var(SESSION_KEY_ENV) {
                Ok(key) => {
                    log::info!("Got session key from env variable.");
                    Key::from(&key.into_bytes())
                },
                Err(_) => {
                    log::info!("Could not find session key in env variable, generate one.");
                    Key::generate()
                }
            }
        }

        #[cfg(feature = "ssr")]
        pub fn get_session_db_key() -> Key {
            match env::var(SESSION_DB_KEY_ENV) {
                Ok(key) => {
                    log::info!("Got session db key from env variable.");
                    Key::from(&key.into_bytes())
                },
                Err(_) => {
                    log::info!("Could not find session db key in env variable, generate one.");
                    Key::generate()
                }
            }
        }

        async fn server_fn_handler(State(app_state): State<AppState>, session: Session, path: Path<String>, headers: HeaderMap, raw_query: RawQuery, request: Request<AxumBody>) -> impl IntoResponse {
            log::info!("{:?}", path);

            handle_server_fns_with_context(path, headers, raw_query, move || {
                provide_context( session.clone());
                provide_context( app_state.pool.clone());
            }, request).await
        }

        async fn leptos_routes_handler(session: Session, State(app_state): State<AppState>, req: Request<AxumBody>) -> Response{
                let handler = leptos_axum::render_app_to_stream_with_context(app_state.leptos_options.clone(),
                move || {
                    provide_context( session.clone());
                    provide_context( app_state.pool.clone());
                },
                || view! {  <App/> }
            );
            handler(req).await.into_response()
        }

        #[tokio::main]
        async fn main() {
            simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

            let pool = get_db_pool().await.unwrap();

            let session_config = SessionConfig::default()
                .with_table_name("sessions")
                // 'Key::generate()' will generate a new key each restart of the server.
                // If you want it to be more permanent then generate and set it to a config file.
                // If with_key() is used it will set all cookies as private, which guarantees integrity, and authenticity.
                .with_key(get_session_key())
                // This is how we would Set a Database Key to encrypt as store our per session keys.
                // This MUST be set in order to use SecurityMode::PerSession.
                .with_database_key(get_session_db_key())
                // This is How you will enable PerSession SessionID Private Cookie Encryption. When enabled it will
                // Encrypt the SessionID and Storage with an Encryption key generated and stored per session.
                // This allows for Key renewing without needing to force the entire Session from being destroyed.
                // This Also helps prevent impersonation attempts.
                .with_security_mode(SecurityMode::PerSession);
            let session_store = SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config).await.unwrap();

            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("could not run SQLx migrations");

            // Setting get_configuration(None) means we'll be using cargo-leptos's env values
            // For deployment these variables are:
            // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
            // Alternately a file can be specified such as Some("Cargo.toml")
            // The file would need to be included with the executable when moved to deployment
            let conf = get_configuration(None).await.unwrap();
            let leptos_options = conf.leptos_options;
            let addr = leptos_options.site_addr;
            let routes = generate_route_list(App);

            let app_state = AppState {
                leptos_options,
                pool: pool.clone(),
            };

            // build our application with a route
            let app = Router::new()
                .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
                .leptos_routes_with_handler(routes, get(leptos_routes_handler))
                .fallback(file_and_error_handler)
                .layer(SessionLayer::new(session_store))
                .with_state(app_state);

            // run our app with hyper
            // `axum::Server` is a re-export of `hyper::Server`
            log::info!("listening on http://{}", &addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }


        async fn get_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&env::var(DB_URL_ENV)?)
                .await
                .with_context(|| format!("Failed to connect to DB"))
        }
    }
    else {
        pub fn main() {
            // no client-side main function
            // unless we want this to work with e.g., Trunk for a purely client-side app
            // see lib.rs for hydration function instead
        }
    }
}

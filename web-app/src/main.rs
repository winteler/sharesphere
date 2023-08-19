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

        use project_web_app::app::*;
        use project_web_app::auth::*;
        use project_web_app::fileserv::file_and_error_handler;
        use project_web_app::state::AppState;

        use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns_with_context};
        use leptos::{log, view, provide_context, get_configuration};

        use sqlx::{PgPool, postgres::{PgPoolOptions}};
        use axum_session::{SessionPgPool, SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::{AuthSessionLayer, AuthConfig};

        use anyhow::{Context};

        async fn server_fn_handler(State(app_state): State<AppState>, auth_session: AuthSession, path: Path<String>, headers: HeaderMap, raw_query: RawQuery, request: Request<AxumBody>) -> impl IntoResponse {
            log!("{:?}", path);

            handle_server_fns_with_context(path, headers, raw_query, move |cx| {
                provide_context(cx, auth_session.clone());
                provide_context(cx, app_state.pool.clone());
            }, request).await
        }

        async fn leptos_routes_handler(auth_session: AuthSession, State(app_state): State<AppState>, req: Request<AxumBody>) -> Response{
                let handler = leptos_axum::render_app_to_stream_with_context(app_state.leptos_options.clone(),
                move |cx| {
                    provide_context(cx, auth_session.clone());
                    provide_context(cx, app_state.pool.clone());
                },
                |cx| view! { cx, <App/> }
            );
            handler(req).await.into_response()
        }

        #[tokio::main]
        async fn main() {
            simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

            let pool = get_db_pool().await.unwrap();

            let session_config = SessionConfig::default()
                .with_table_name("sessions");
            let auth_config = AuthConfig::<String>::default().with_anonymous_user_id(Some(String::default()));
            let session_store = SessionStore::<SessionPgPool>::new(Some(pool.clone().into()), session_config).await.unwrap();

            //Create the Database table for storing our Session Data.
            session_store.initiate().await.unwrap();

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
            let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

            let app_state = AppState {
                leptos_options,
                pool: pool.clone(),
            };

            // build our application with a route
            let app = Router::new()
                .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
                .leptos_routes_with_handler(routes, get(leptos_routes_handler))
                .fallback(file_and_error_handler)
                .layer(AuthSessionLayer::<User, String, SessionPgPool, PgPool>::new(Some(pool)).with_config(auth_config))
                .layer(SessionLayer::new(session_store))
                .with_state(app_state);

            // run our app with hyper
            // `axum::Server` is a re-export of `hyper::Server`
            log!("listening on http://{}", &addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }


        async fn get_db_pool() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
            PgPoolOptions::new()
                .max_connections(5)
                .connect("postgres://project:project@localhost:5435/project")
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

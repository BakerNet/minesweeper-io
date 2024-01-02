use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
    use axum::{
        response::{Response, IntoResponse},
        Router,
        routing::get,
        extract::{Path, State, RawQuery},
        body::Body,
        http::{Request, header::HeaderMap}
    };
    use axum_login::tower_sessions::Session;
    use leptos::*;
    use leptos_axum::{LeptosRoutes, *};

    use minesweeper_web::backend::users::AuthSession;
    use minesweeper_web::fileserv::file_and_error_handler;
    use minesweeper_web::{app, backend};

    async fn server_fn_handler(
        State(app_state): State<app::AppState>,
        auth_session: AuthSession,
        session: Session,
        path: Path<String>,
        headers: HeaderMap,
        raw_query: RawQuery,
        request: Request<Body>
    ) -> impl IntoResponse {
        handle_server_fns_with_context(path, headers, raw_query, move || {
            provide_context(auth_session.clone());
            provide_context(session.clone());
            provide_context(app_state.clone());
        }, request).await
    }

    async fn leptos_routes_handler(
        State(app_state): State<app::AppState>,
        auth_session: AuthSession,
        session: Session,
        req: Request<Body>,
    ) -> Response{
        let handler = leptos_axum::render_route_with_context(
            app_state.leptos_options.clone(),
            app_state.routes.clone(),
            move || {
                provide_context(auth_session.clone());
                provide_context(session.clone());
                provide_context(app_state.clone());
            },
            app::App
        );
        handler(req).await.into_response()
    }

    #[tokio::main]
    async fn main() {
        simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

        // Setting get_configuration(None) means we'll be using cargo-leptos's env values
        // For deployment these variables are:
        // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
        // Alternately a file can be specified such as Some("Cargo.toml")
        // The file would need to be included with the executable when moved to deployment
        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(app::App);
        let app_state = app::AppState {
            leptos_options,
            routes: routes.clone(),
        };
        let backend_app = backend::App::new()
            .await
            .expect("Couldn't create backend app");

        // build our application with a route
        let app = Router::new()
            .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
            .leptos_routes_with_handler(routes, get(leptos_routes_handler))
            .fallback(file_and_error_handler);
        let app = backend_app.extend_router(app)
            .with_state(app_state);

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        log::info!("listening on http://{}", &addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

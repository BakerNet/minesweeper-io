use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{FromRef, Path, RawQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    BoxError, Router,
};
use axum_login::{
    tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use http::{HeaderMap, Request};
use leptos::*;
use leptos_axum::*;
use leptos_router::*;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sqlx::SqlitePool;
use std::{env, net::SocketAddr};
use time::Duration;
use tower::ServiceBuilder;
use tower_sessions::{Session, SqliteStore};

use super::{auth, fileserv::file_and_error_handler, game_manager, users};
use crate::{app, app::auth::OAuthTarget};

use super::{auth::REDIRECT_URL, game_manager::GameManager, users::AuthSession};

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub routes: Vec<RouteListing>,
    pub game_manager: GameManager,
}

pub struct App {
    pub db: SqlitePool,
    pub google_client: BasicClient,
    pub reddit_client: BasicClient,
    pub github_client: BasicClient,
    pub session_store: SqliteStore,
}

fn oauth_client(target: OAuthTarget) -> Result<BasicClient, Box<dyn std::error::Error>> {
    let (id_key, secret_key, auth_url, token_url) = match target {
        OAuthTarget::GOOGLE => (
            "GOOGLE_CLIENT_ID",
            "GOOGLE_CLIENT_SECRET",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        OAuthTarget::REDDIT => (
            "REDDIT_CLIENT_ID",
            "REDDIT_CLIENT_SECRET",
            "https://www.reddit.com/api/v1/authorize",
            "https://www.reddit.com/api/v1/access_token",
        ),
        OAuthTarget::GITHUB => (
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
    };
    let client_id = env::var(id_key)
        .map(ClientId::new)
        .expect(&format!("{} should be provided.", id_key));
    let client_secret = env::var(secret_key)
        .map(ClientSecret::new)
        .expect(&format!("{} should be provided.", secret_key));
    let redirect_host = env::var("REDIRECT_HOST")
        .map(String::from)
        .expect("REDIRECT_HOST should be provided");

    let auth_url = AuthUrl::new(auth_url.to_string())?;
    let token_url = TokenUrl::new(token_url.to_string())?;
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(redirect_host + REDIRECT_URL)?);

    Ok(client)
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    session: Session,
    path: Path<String>,
    headers: HeaderMap,
    raw_query: RawQuery,
    request: http::Request<Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        path,
        headers,
        raw_query,
        move || {
            provide_context(auth_session.clone());
            provide_context(session.clone());
            provide_context(app_state.game_manager.clone());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    session: Session,
    req: Request<Body>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(session.clone());
            provide_context(app_state.game_manager.clone());
        },
        app::App,
    );
    handler(req).await.into_response()
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv()?;

        let google_client = oauth_client(OAuthTarget::GOOGLE)?;
        let reddit_client = oauth_client(OAuthTarget::REDDIT)?;
        let github_client = oauth_client(OAuthTarget::GITHUB)?;

        let db_url = env::var("DATABASE_URL")
            .map(String::from)
            .expect("DATABASE_URL should be provided");

        let db = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!().run(&db).await?;

        let session_store = SqliteStore::new(db.clone());
        session_store
            .migrate()
            .await
            .expect("Migrations for session store should work");

        Ok(Self {
            db,
            google_client,
            reddit_client,
            github_client,
            session_store,
        })
    }

    pub async fn router(self) -> (Router, SocketAddr) {
        // Setting get_configuration(None) means we'll be using cargo-leptos's env values
        // For deployment these variables are:
        // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
        // Alternately a file can be specified such as Some("Cargo.toml")
        // The file would need to be included with the executable when moved to deployment
        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(app::App);
        let game_manager = GameManager::new(self.db.clone());

        let app_state = AppState {
            leptos_options,
            routes: routes.clone(),
            game_manager,
        };

        // Session layer.
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let session_layer = SessionManagerLayer::new(self.session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = users::Backend::new(
            self.db.clone(),
            self.google_client,
            self.reddit_client,
            self.github_client,
        );
        let auth_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|_: BoxError| async {
                StatusCode::BAD_REQUEST
            }))
            .layer(AuthManagerLayerBuilder::new(backend, session_layer).build());

        // build our application with a route
        let app = Router::new()
            .route(
                "/api/*fn_name",
                get(server_fn_handler).post(server_fn_handler),
            )
            .leptos_routes_with_handler(routes, get(leptos_routes_handler))
            .fallback(file_and_error_handler)
            .merge(auth::router())
            .merge(game_manager::router())
            .layer(auth_service)
            .with_state(app_state);
        (app, addr)
    }
}

// TODO - migrate this config
//
// use axum::{
//     http::Method,
//     routing::{get, post},
//     Router,
// };
// use server::{
//     create_game, game_manager::GameManager, index, play_game, websocket_handler, AppState,
// };
// use std::{net::SocketAddr, sync::Arc};
// use tower_http::cors::{Any, CorsLayer};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//
// #[tokio::main]
// async fn main() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::EnvFilter::try_from_default_env()
//                 .unwrap_or_else(|_| "example_chat=trace".into()),
//         )
//         .with(tracing_subscriber::fmt::layer())
//         .init();
//
//     // Set up application state for use with with_state().
//
//     let app_state = Arc::new(AppState {
//         game_manager: GameManager::new(),
//     });
//
//     let cors = CorsLayer::new()
//         .allow_origin(Any)
//         .allow_methods([Method::GET, Method::POST]);
//     let app = Router::new()
//         .route("/api/new", post(create_game))
//         .route("/api/play", post(play_game))
//         .route("/api/websocket", get(websocket_handler))
//         .route("/public/*path", get(index))
//         .route("/", get(index))
//         .route("/*path", get(index))
//         .layer(cors)
//         .with_state(app_state);
//
//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     tracing::debug!("listening on {}", addr);
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }

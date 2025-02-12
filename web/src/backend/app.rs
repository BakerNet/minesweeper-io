use anyhow::Result;
use axum::{
    body::Body,
    extract::{FromRef, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_login::AuthManagerLayerBuilder;
use http::Request;
use leptos::prelude::*;
use leptos_axum::*;
use oauth2::basic::BasicClient;
use sqlx::SqlitePool;
use std::{env, net::SocketAddr};
use time::Duration;
use tower_sessions::{
    cookie::SameSite, session_store, ExpiredDeletion, Expiry, Session, SessionManagerLayer,
};
use tower_sessions_sqlx_store::SqliteStore;

use game_manager::{models::Game, websocket_handler, ExtractGameManager, GameManager};
use web_auth::{oauth_callback, oauth_client, AuthSession, Backend, OAuthTarget, REDIRECT_URL};

use crate::app::{shell, App as FrontendApp};

use super::fileserv::file_and_error_handler;

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub routes: Vec<AxumRouteListing>,
    pub game_manager: GameManager,
}

impl ExtractGameManager for AppState {
    fn game_manager(&self) -> GameManager {
        self.game_manager.clone()
    }
}

pub fn services_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/api/websocket/game/:id",
            get(websocket_handler::<AppState>),
        )
        .route(REDIRECT_URL, get(oauth_callback))
}

pub struct App {
    pub db: SqlitePool,
    pub google_client: BasicClient,
    pub reddit_client: BasicClient,
    pub github_client: BasicClient,
    pub session_store: SqliteStore,
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    session: Session,
    request: http::Request<Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(session.clone());
            provide_context(app_state.game_manager());
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
    let routes = app_state.routes.clone();
    let game_manager = app_state.game_manager.clone();
    let options = app_state.leptos_options.clone();
    let handler = leptos_axum::render_route_with_context(
        routes,
        move || {
            provide_context(auth_session.clone());
            provide_context(session.clone());
            provide_context(game_manager.clone());
        },
        move || shell(options.clone()),
    );
    handler(State(app_state), req).await.into_response()
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv()?;

        let google_client = oauth_client(OAuthTarget::Google)?;
        let reddit_client = oauth_client(OAuthTarget::Reddit)?;
        let github_client = oauth_client(OAuthTarget::Github)?;

        let db_url = env::var("DATABASE_URL")
            .map(String::from)
            .expect("DATABASE_URL should be provided");

        let db = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!("../migrations").run(&db).await?;
        // Close out any dangling games from before restart
        Game::set_all_games_completed(&db).await?;

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

    pub fn start_session_cleanup(
        &self,
    ) -> tokio::task::JoinHandle<Result<(), session_store::Error>> {
        let deletion_task = tokio::task::spawn(
            self.session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );
        deletion_task
    }

    pub async fn router(self) -> (Router, SocketAddr) {
        // Setting get_configuration(None) means we'll be using cargo-leptos's env values
        // For deployment these variables are:
        // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
        // Alternately a file can be specified such as Some("Cargo.toml")
        // The file would need to be included with the executable when moved to deployment
        let conf = get_configuration(None).unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(FrontendApp);
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
            .with_expiry(Expiry::OnInactivity(Duration::days(30)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(
            self.db.clone(),
            self.google_client,
            self.reddit_client,
            self.github_client,
        );
        let auth_service = AuthManagerLayerBuilder::new(backend, session_layer).build();

        // build our application with a route
        let app = Router::new()
            .route(
                "/api/*fn_name",
                get(server_fn_handler).post(server_fn_handler),
            )
            .leptos_routes_with_handler(routes, get(leptos_routes_handler))
            .fallback(file_and_error_handler)
            .merge(services_router())
            .layer(auth_service)
            .with_state(app_state);
        (app, addr)
    }
}

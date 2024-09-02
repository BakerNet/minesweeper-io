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
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sqlx::SqlitePool;
use std::{env, net::SocketAddr};
use time::Duration;
use tower_sessions::{
    cookie::SameSite, session_store, ExpiredDeletion, Expiry, Session, SessionManagerLayer,
};
use tower_sessions_sqlx_store::SqliteStore;

use crate::{
    app::{shell, App as FrontendApp, OAuthTarget},
    models::game::Game,
};

use super::{
    auth, auth::REDIRECT_URL, fileserv::file_and_error_handler, game_manager::GameManager, users,
    users::AuthSession, websocket,
};

/// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub routes: Vec<AxumRouteListing>,
    pub game_manager: GameManager,
}

pub struct App {
    pub db: SqlitePool,
    pub google_client: BasicClient,
    pub reddit_client: BasicClient,
    pub github_client: BasicClient,
    pub session_store: SqliteStore,
}

fn oauth_client(target: OAuthTarget) -> Result<BasicClient> {
    let (id_key, secret_key, auth_url, token_url) = match target {
        OAuthTarget::Google => (
            "GOOGLE_CLIENT_ID",
            "GOOGLE_CLIENT_SECRET",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        OAuthTarget::Reddit => (
            "REDDIT_CLIENT_ID",
            "REDDIT_CLIENT_SECRET",
            "https://www.reddit.com/api/v1/authorize",
            "https://www.reddit.com/api/v1/access_token",
        ),
        OAuthTarget::Github => (
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
    };
    let client_id = env::var(id_key)
        .map(ClientId::new)
        .unwrap_or_else(|_| panic!("{} should be provided.", id_key));
    let client_secret = env::var(secret_key)
        .map(ClientSecret::new)
        .unwrap_or_else(|_| panic!("{} should be provided.", secret_key));
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
    request: http::Request<Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
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
    handler(req).await.into_response()
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
        sqlx::migrate!().run(&db).await?;
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
        let backend = users::Backend::new(
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
            .merge(auth::router())
            .merge(websocket::router())
            .layer(auth_service)
            .with_state(app_state);
        (app, addr)
    }
}

#[cfg(feature = "ssr")]
use minesweeper_web::backend::App;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    let backend_app = App::new().await.expect("Couldn't create backend app");
    let session_cleanup_task = backend_app.start_session_cleanup();
    let (app, addr) = backend_app.router().await;

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    session_cleanup_task.await??;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}

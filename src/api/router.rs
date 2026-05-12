use crate::{base::config::Config, bot, logs};
use axum::{Router, routing::post};
use std::fs;
use tokio::net::UnixListener;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run(cancel_token: CancellationToken) {
    let config = Config::get();

    fs::remove_file(&config.sock_path).ok();
    let listener = UnixListener::bind(&config.sock_path)
        .unwrap_or_else(|e| panic!("Failed to bind unix socket {:?}: {}", config.sock_path, e));

    info!("Unix socket listening on {:?}", config.sock_path);

    let mut router = Router::new().route("/log", post(logs::server::handle_log));

    if config.bot.en && config.bot.webhook_url.is_some() {
        let bot_router = bot::router::router(cancel_token.clone()).await;
        router = router.merge(bot_router);
    }

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            cancel_token.cancelled().await;
        })
        .await
        .unwrap_or_else(|e| panic!("Unix socket server failed: {}", e));

    fs::remove_file(&config.sock_path).ok();
    info!("Unix socket server stopped");
}

use crate::{base::config::Config, bot};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run(cancel_token: CancellationToken) {
    let config = Config::get();

    // Only start webhook server if webhook_url is configured
    if !config.bot.en || config.bot.webhook_url.is_none() {
        return;
    }

    let bot_router = bot::router::router(cancel_token.clone()).await;

    let port = config.bot.webhook_port.unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to {addr}: {err}"));

    axum::serve(listener, bot_router)
        .with_graceful_shutdown(async move {
            cancel_token.cancelled().await;
        })
        .await
        .unwrap_or_else(|err| panic!("Failed to serve webhook: {err}"));

    info!("Webhook server stopped");
}

use crate::{base::config::Config, bot::bot};
use axum::Router;
use std::net::{Ipv4Addr, SocketAddr};
use teloxide::{
    prelude::*,
    update_listeners::webhooks::{Options, axum_to_router},
};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run(cancel_token: CancellationToken) {
    let (bot, handler) = bot::init();
    let mut dispatcher = Dispatcher::builder(bot, handler).build();
    let shutdown_token = dispatcher.shutdown_token();

    tokio::spawn(async move {
        info!("Bot dispatcher starting");
        dispatcher.dispatch().await;
    });

    cancel_token.cancelled().await;
    shutdown_token.shutdown().ok();
    info!("Bot dispatcher stopped");
}

pub async fn router(cancel_token: CancellationToken) -> Router {
    let (bot, handler) = bot::init();
    let mut dispatcher = Dispatcher::builder(bot.clone(), handler).build();
    let shutdown_token = dispatcher.shutdown_token();

    let config = Config::get();
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let url = config.bot.webhook_url.as_ref().unwrap();
    let opt = Options::new(addr, url.clone());
    let (listener, _, router) = axum_to_router(bot.clone(), opt)
        .await
        .unwrap_or_else(|err| panic!("Failed to set up bot webhook: {err}"));

    tokio::spawn(async move {
        info!("Starting bot webhook server at {url}");
        tokio::join!(
            async {
                let error_handler = LoggingErrorHandler::with_custom_text("Axum bot error:");
                dispatcher
                    .dispatch_with_listener(listener, error_handler)
                    .await;
            },
            async {
                cancel_token.cancelled().await;
                bot.delete_webhook().send().await.ok();
                shutdown_token.shutdown().ok();
            }
        );
        info!("Bot webhook server shutting down");
    });

    router
}

use admin::{
    api, backup,
    base::{config::Config, log, signal},
    bot,
    kv::kv::KvStore,
    services,
};
use tokio_util::sync::CancellationToken;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = Config::get();
    let token = CancellationToken::new();
    let log_rx = log::init();

    // Initialize KV store
    KvStore::global();

    tokio::join!(
        signal::run(token.clone()),
        api::router::run(token.clone()),
        async {
            if let Some(log_rx) = log_rx {
                log::run(log_rx, token.clone()).await;
            }
        },
        async {
            // Polling mode: run when webhook_url is not set
            if config.bot.en && config.bot.webhook_url.is_none() {
                bot::router::run(token.clone()).await;
            }
        },
        services::scheduler::run(token.clone()),
        backup::scheduler::run(token.clone()),
    );
}

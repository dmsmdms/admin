use admin::{
    api, backup,
    base::{config::Config, log, signal},
    bot,
    kv::kv::KvStore,
    logs, services,
};
use tokio_util::sync::CancellationToken;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = Config::get();
    let token = CancellationToken::new();
    let log_rx = log::init();

    // Initialize KV store
    KvStore::global();

    let log_sock = config.admin.as_ref().map(|a| a.log_sock_path.clone());

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
        async {
            if let Some(sock) = &log_sock {
                logs::server::run(sock, token.clone()).await;
            }
        },
        services::scheduler::run(token.clone()),
        backup::scheduler::run(token.clone()),
    );
}

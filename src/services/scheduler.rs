use crate::{
    base::config::Config,
    bot::bot::send_message,
    kv::{bot::Permission, kv::KvStore},
    services::flatdb,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub async fn run(cancel_token: CancellationToken) {
    let config = Config::get();

    let flatdb_service = config
        .services
        .iter()
        .find(|s| s.name == "flatdb" && s.enabled);

    let service = match flatdb_service {
        Some(s) => s.clone(),
        None => return,
    };

    let interval_secs = service.mon_interval_sec;
    info!("flatdb monitor starting, interval={}s, sock={:?}", interval_secs, service.sock_path);

    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                check_flatdb(&service.sock_path).await;
            }
            _ = cancel_token.cancelled() => {
                info!("flatdb monitor stopped");
                break;
            }
        }
    }
}

async fn check_flatdb(sock_path: &std::path::Path) {
    match flatdb::request_status(sock_path).await {
        Ok(status) => {
            if !status.ok {
                let msg = format!("⚠️ <b>flatdb</b> returned ok=false\n{}", flatdb::format_status(&status));
                broadcast_to_perm(Permission::Status, msg).await;
            }
        }
        Err(e) => {
            error!("flatdb status check failed: {}", e);
            let msg = format!("🚨 <b>flatdb</b> is not responding\n<code>{}</code>", e);
            broadcast_to_perm(Permission::Status, msg).await;
        }
    }
}

async fn broadcast_to_perm(perm: Permission, msg: String) {
    // Collect recipients before any await to avoid holding non-Send KV iter
    let chat_ids = {
        let kv = KvStore::global();
        let rtxn = match kv.env.read_txn() {
            Ok(t) => t,
            Err(e) => {
                error!("KV read txn failed: {}", e);
                return;
            }
        };
        let iter = match kv.chat_auth.iter(&rtxn) {
            Ok(i) => i,
            Err(e) => {
                error!("KV iter failed: {}", e);
                return;
            }
        };
        iter.filter_map(|r| r.ok())
            .filter(|(_, auth)| auth.perms.contains(&perm))
            .map(|(id, _)| id)
            .collect::<Vec<_>>()
    };

    for chat_id in chat_ids {
        if let Err(e) = send_message(chat_id, msg.clone()).await {
            error!("send_message to {} failed: {}", chat_id, e);
        }
    }
}

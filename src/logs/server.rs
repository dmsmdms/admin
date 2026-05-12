use crate::{
    bot::bot::send_message,
    kv::{bot::Permission, kv::KvStore},
    logs::formatter::{self, Log},
};
use axum::{body::Bytes, http::StatusCode};
use tracing::{error, warn};

pub async fn handle_log(body: Bytes) -> StatusCode {
    for line in body.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        match serde_json::from_slice::<Log>(line) {
            Ok(log) => {
                let text = formatter::format_log(&log);
                broadcast_to_perm(Permission::Log, text).await;
            }
            Err(e) => {
                warn!(
                    "Failed to parse log entry: {} — {:?}",
                    e,
                    String::from_utf8_lossy(line)
                );
            }
        }
    }
    StatusCode::OK
}

async fn broadcast_to_perm(perm: Permission, msg: String) {
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


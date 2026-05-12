use crate::{
    bot::bot::send_message,
    kv::{bot::Permission, kv::KvStore},
    logs::formatter::{self, Log},
};
use std::fs;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub async fn run(sock_path: &std::path::Path, cancel_token: CancellationToken) {
    fs::remove_file(sock_path).ok();

    let listener = match UnixListener::bind(sock_path) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind log socket {:?}: {}", sock_path, e);
            return;
        }
    };
    info!("Log server listening on {:?}", sock_path);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        // current_thread runtime — spawn_local is fine
                        tokio::task::spawn_local(handle_connection(stream));
                    }
                    Err(e) => {
                        warn!("Log socket accept error: {}", e);
                    }
                }
            }
            _ = cancel_token.cancelled() => {
                info!("Log server stopped");
                fs::remove_file(sock_path).ok();
                break;
            }
        }
    }
}

async fn handle_connection(mut stream: tokio::net::UnixStream) {
    let mut buf = Vec::new();
    if let Err(e) = stream.read_to_end(&mut buf).await {
        warn!("Log read error: {}", e);
        return;
    }

    // Each connection may send one or multiple newline-delimited JSON log entries
    for line in buf.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        match serde_json::from_slice::<Log>(line) {
            Ok(log) => {
                let text = formatter::format_log(&log);
                broadcast_to_perm(Permission::Log, text).await;
            }
            Err(e) => {
                warn!("Failed to parse log entry: {} — {:?}", e, String::from_utf8_lossy(line));
            }
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

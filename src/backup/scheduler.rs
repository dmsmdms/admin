use crate::{
    backup::{sender, splitter},
    base::config::Config,
    bot::bot::send_message,
    kv::{
        kv::KvStore,
        meta::{MetaKey, MetaValue},
    },
};
use chrono::{Local, NaiveTime, Timelike};
use std::path::Path;
use teloxide::{prelude::*, types::InputFile};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub async fn run(cancel_token: CancellationToken) {
    let config = Config::get();
    let admin = match &config.admin {
        Some(a) => a,
        None => return,
    };

    let backup_time = match NaiveTime::parse_from_str(&admin.backup_time, "%H:%M") {
        Ok(t) => t,
        Err(e) => {
            error!("Invalid backup_time '{}': {}", admin.backup_time, e);
            return;
        }
    };

    info!("Backup scheduler starting, time={}", admin.backup_time);

    loop {
        let now = Local::now().time();
        let target = backup_time;

        // Seconds until next backup window
        let secs_until = {
            let now_secs = now.num_seconds_from_midnight();
            let target_secs = target.num_seconds_from_midnight();
            if target_secs > now_secs {
                (target_secs - now_secs) as u64
            } else {
                (86400 - now_secs + target_secs) as u64
            }
        };

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(secs_until)) => {
                run_backups().await;
            }
            _ = cancel_token.cancelled() => {
                info!("Backup scheduler stopped");
                break;
            }
        }
    }
}

async fn run_backups() {
    let config = Config::get();

    let admin = match &config.admin {
        Some(a) => a,
        None => return,
    };

    let backup_chat_id = admin.backup_chat_id;
    let tg_token = &config.bot.token;

    for service in config.services.iter().filter(|s| s.enabled) {
        info!("Running backup for service '{}'", service.name);

        // Get last backup timestamp from KV
        let last_ts = match KvStore::get_meta(MetaKey::LastBackupTs) {
            Ok(Some(MetaValue::LastBackupTs(ts))) => ts,
            _ => 0,
        };

        // Request backup from service
        let backup_result = sender::request_backup(&service.sock_path, last_ts).await;
        let resp = match backup_result {
            Ok(r) => r,
            Err(e) => {
                error!("Backup request failed for '{}': {}", service.name, e);
                let _ = send_message(
                    backup_chat_id,
                    format!("❌ Backup failed for <b>{}</b>\n<code>{}</code>", service.name, e),
                )
                .await;
                continue;
            }
        };

        if resp.paths.is_empty() {
            info!("No backup files returned for '{}'", service.name);
            continue;
        }

        let bot = Bot::new(tg_token);
        let now_ts = chrono::Utc::now().timestamp();

        // Send each file (split into 50MB chunks) to backup chat
        for path_str in &resp.paths {
            let path = Path::new(path_str);
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("backup");

            match splitter::split_file(path).await {
                Ok(chunks) => {
                    let total = chunks.len();
                    for (i, chunk) in chunks.into_iter().enumerate() {
                        let part_name = if total > 1 {
                            format!("{}.part{}", filename, i + 1)
                        } else {
                            filename.to_string()
                        };

                        if let Err(e) = bot
                            .send_document(
                                ChatId(backup_chat_id),
                                InputFile::memory(chunk).file_name(part_name),
                            )
                            .caption(format!(
                                "📦 <b>{}</b>\n🗂 {}\n{}/{}",
                                service.name,
                                filename,
                                i + 1,
                                total
                            ))
                            .await
                        {
                            error!("Failed to send backup chunk: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read backup file {:?}: {}", path, e);
                    let _ = send_message(
                        backup_chat_id,
                        format!("⚠️ Could not read backup file <code>{}</code>: {}", path_str, e),
                    )
                    .await;
                }
            }
        }

        // Update last backup timestamp in KV
        if let Err(e) = KvStore::set_meta(MetaValue::LastBackupTs(now_ts)) {
            error!("Failed to save backup timestamp: {}", e);
        }
    }
}

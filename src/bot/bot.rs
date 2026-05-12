use crate::{
    auth::{tokens::get_perms_for_token, validate_token, TokenType},
    base::config::Config,
    kv::{bot::Permission, kv::KvStore},
    services::flatdb,
};
use std::{error::Error, sync::OnceLock};
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::UpdateHandler,
    dptree,
    errors::RequestError,
    macros::BotCommands,
    prelude::*,
    types::ParseMode,
};

static BOT: OnceLock<DefaultParseMode<Bot>> = OnceLock::new();

pub fn init() -> (DefaultParseMode<Bot>, UpdateHandler<RequestError>) {
    let config = Config::get();
    let bot = Bot::new(&config.bot.token).parse_mode(ParseMode::Html);
    BOT.set(bot.clone()).expect("Failed to set bot");

    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(|bot: Bot, msg: Message, cmd: Command| async move {
                handle_command(bot, msg, cmd).await
            }),
    );

    (bot, handler)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Start using the bot")]
    Start,
    #[command(description = "Authenticate with token: /auth <token>")]
    Auth { token: String },
    #[command(description = "Check your permissions")]
    PermsCheck,
    #[command(description = "Get status of all services")]
    Status,
    #[command(rename = "flatdb-status", description = "Get flatdb service status")]
    FlatdbStatus,
}

// Helper: convert KvStore results to String errors before any await point
fn kv_has_perm(chat_id: i64, perm: Permission) -> Result<bool, String> {
    KvStore::has_perm(chat_id, perm).map_err(|e| e.to_string())
}

fn kv_get_auth(chat_id: i64) -> Result<Option<crate::kv::bot::ChatAuth>, String> {
    KvStore::get_chat_auth(chat_id).map_err(|e| e.to_string())
}

fn kv_set_auth(chat_id: i64, auth: crate::kv::bot::ChatAuth) -> Result<(), String> {
    KvStore::set_chat_auth(chat_id, auth).map_err(|e| e.to_string())
}

async fn handle_command(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;

    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "👋 Welcome to <b>Admin Bot</b>!\n\nService monitoring and backup management.\n\nUse /auth &lt;token&gt; to authenticate.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }

        Command::Auth { token } => {
            match validate_token(&token) {
                Some(token_type) => {
                    let perms = get_perms_for_token(token_type);
                    let auth = crate::kv::bot::ChatAuth {
                        chat_id,
                        perms: perms.clone(),
                    };
                    let save_result = kv_set_auth(chat_id, auth);
                    match save_result {
                        Ok(_) => {
                            // Store backup chat ID if this is a backup token
                            if token_type == TokenType::Backup {
                                if let Err(e) = KvStore::set_meta(
                                    crate::kv::meta::MetaValue::BackupChatId(chat_id),
                                ) {
                                    tracing::error!("Failed to set backup chat ID: {}", e);
                                }
                            }

                            let perms_str = perms
                                .iter()
                                .map(|p| format!(" • {p:?}"))
                                .collect::<Vec<_>>()
                                .join("\n");
                            bot.send_message(
                                msg.chat.id,
                                format!("✅ Authentication successful!\n\n📋 Permissions:\n{perms_str}"),
                            )
                            .parse_mode(ParseMode::Html)
                            .await?;
                        }
                        Err(err_str) => {
                            bot.send_message(msg.chat.id, format!("❌ Error saving auth: {err_str}"))
                                .await?;
                        }
                    }
                }
                None => {
                    bot.send_message(msg.chat.id, "❌ Invalid token").await?;
                }
            }
        }

        Command::PermsCheck => {
            let auth_result = kv_get_auth(chat_id);
            match auth_result {
                Ok(Some(auth)) => {
                    let perms_str = auth
                        .perms
                        .iter()
                        .map(|p| format!(" • {p:?}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    bot.send_message(msg.chat.id, format!("📋 Your permissions:\n{perms_str}"))
                        .parse_mode(ParseMode::Html)
                        .await?;
                }
                Ok(None) => {
                    bot.send_message(
                        msg.chat.id,
                        "⚠️ Not authenticated. Use /auth &lt;token&gt;",
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
                Err(err_str) => {
                    bot.send_message(msg.chat.id, format!("❌ Error: {err_str}")).await?;
                }
            }
        }

        Command::Status => {
            let perm_result = kv_has_perm(chat_id, Permission::Status);
            match perm_result {
                Ok(false) => {
                    bot.send_message(msg.chat.id, "❌ Permission denied. Required: Status")
                        .await?;
                    return Ok(());
                }
                Err(err_str) => {
                    bot.send_message(msg.chat.id, format!("❌ Error: {err_str}")).await?;
                    return Ok(());
                }
                Ok(true) => {}
            }
            let config = Config::get();
            let lines: Vec<String> = config
                .services
                .iter()
                .map(|s| {
                    let icon = if s.enabled { "🟢" } else { "⚫" };
                    format!(
                        "{icon} <b>{}</b> — {}",
                        s.name,
                        if s.enabled { "enabled" } else { "disabled" }
                    )
                })
                .collect();
            let text = if lines.is_empty() {
                "📊 No services configured.".to_string()
            } else {
                format!("📊 <b>Services</b>\n{}", lines.join("\n"))
            };
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }

        Command::FlatdbStatus => {
            let perm_result = kv_has_perm(chat_id, Permission::Status);
            match perm_result {
                Ok(false) => {
                    bot.send_message(msg.chat.id, "❌ Permission denied. Required: Status")
                        .await?;
                    return Ok(());
                }
                Err(err_str) => {
                    bot.send_message(msg.chat.id, format!("❌ Error: {err_str}")).await?;
                    return Ok(());
                }
                Ok(true) => {}
            }
            let config = Config::get();
            let svc = config
                .services
                .iter()
                .find(|s| s.name == "flatdb" && s.enabled)
                .map(|s| s.sock_path.clone());
            match svc {
                None => {
                    bot.send_message(
                        msg.chat.id,
                        "⚫ flatdb service is disabled or not configured.",
                    )
                    .await?;
                }
                Some(sock_path) => {
                    bot.send_message(msg.chat.id, "⏳ Checking flatdb…").await?;
                    match flatdb::request_status(&sock_path).await {
                        Ok(status) => {
                            let text = flatdb::format_status(&status);
                            bot.send_message(msg.chat.id, text)
                                .parse_mode(ParseMode::Html)
                                .await?;
                        }
                        Err(e) => {
                            bot.send_message(
                                msg.chat.id,
                                format!("🚨 <b>flatdb</b> not responding\n<code>{e}</code>"),
                            )
                            .parse_mode(ParseMode::Html)
                            .await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn send_message(chat_id: i64, text: String) -> Result<(), Box<dyn Error>> {
    let bot = BOT.get().expect("Bot not initialized");
    bot.send_message(ChatId(chat_id), text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

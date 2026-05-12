use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum LogType {
    Error,
    Warning,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub log_type: LogType,
    pub ts_sec: i64,
    pub ts_msec: u32,
    pub file: String,
    pub msg: String,
}

/// Format a Log into a pretty Telegram HTML message
pub fn format_log(log: &Log) -> String {
    let (icon, label) = match log.log_type {
        LogType::Error => ("🚨", "ERROR"),
        LogType::Warning => ("⚠️", "WARNING"),
    };

    let ts = DateTime::<Utc>::from_timestamp(log.ts_sec, log.ts_msec * 1_000_000)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| log.ts_sec.to_string());

    format!(
        "{} <b>{}</b>\n🕒 {}\n📦 <code>{}</code>\n💬 {}",
        icon, label, ts, log.file, log.msg
    )
}

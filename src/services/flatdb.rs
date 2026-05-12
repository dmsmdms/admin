use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FlatStat {
    pub parse_ts: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FlatdbStatus {
    pub ok: bool,
    pub flat: Option<FlatStat>,
    pub flat_rent: Option<FlatStat>,
}

/// Send HTTP-like GET /status request over unix socket and parse response
pub async fn request_status(sock_path: &std::path::Path) -> Result<FlatdbStatus, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let mut stream = UnixStream::connect(sock_path)
        .await
        .map_err(|e| format!("connect: {e}"))?;

    stream
        .write_all(b"GET /status\r\n")
        .await
        .map_err(|e| format!("write: {e}"))?;
    stream.shutdown().await.map_err(|e| format!("shutdown: {e}"))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(|e| format!("read: {e}"))?;

    serde_json::from_slice(&buf).map_err(|e| format!("parse JSON ({e}): {}", String::from_utf8_lossy(&buf)))
}

/// Format FlatdbStatus into a pretty Telegram HTML message
pub fn format_status(status: &FlatdbStatus) -> String {
    use chrono::{DateTime, Utc};

    let status_icon = if status.ok { "✅" } else { "❌" };
    let mut msg = format!("{} <b>flatdb</b>\n", status_icon);

    let fmt_ts = |ts: i64| -> String {
        DateTime::<Utc>::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| ts.to_string())
    };

    if let Some(flat) = &status.flat {
        msg.push_str(&format!("📄 <b>flat</b> last parsed: <code>{}</code>\n", fmt_ts(flat.parse_ts)));
    }
    if let Some(flat_rent) = &status.flat_rent {
        msg.push_str(&format!(
            "🏘 <b>flat-rent</b> last parsed: <code>{}</code>\n",
            fmt_ts(flat_rent.parse_ts)
        ));
    }

    msg
}

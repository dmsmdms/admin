use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub struct BackupResponse {
    pub paths: Vec<String>,
}

/// Send /backup {last_ts} over unix socket and return list of created file paths
pub async fn request_backup(sock_path: &Path, last_ts: i64) -> Result<BackupResponse, String> {
    let mut stream = UnixStream::connect(sock_path)
        .await
        .map_err(|e| format!("connect: {e}"))?;

    let req = format!("GET /backup?ts={last_ts}\r\n");
    stream
        .write_all(req.as_bytes())
        .await
        .map_err(|e| format!("write: {e}"))?;
    stream.shutdown().await.map_err(|e| format!("shutdown: {e}"))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(|e| format!("read: {e}"))?;

    // Expect JSON array of file paths: ["path1", "path2", ...]
    let paths: Vec<String> =
        serde_json::from_slice(&buf).map_err(|e| format!("parse JSON: {e}"))?;
    Ok(BackupResponse { paths })
}

use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

const CHUNK_SIZE: usize = 50 * 1024 * 1024; // 50 MB

/// Split a file into chunks of up to 50 MB.
/// Returns in-memory byte chunks ready for upload.
pub async fn split_file(path: &Path) -> Result<Vec<Vec<u8>>, String> {
    let file =
        File::open(path).await.map_err(|e| format!("open {:?}: {e}", path))?;
    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();

    loop {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let mut n = 0;

        while n < CHUNK_SIZE {
            match reader.read(&mut chunk[n..]).await {
                Ok(0) => break,
                Ok(read) => n += read,
                Err(e) => return Err(format!("read {:?}: {e}", path)),
            }
        }

        if n == 0 {
            break;
        }
        chunk.truncate(n);
        chunks.push(chunk);

        if n < CHUNK_SIZE {
            break;
        }
    }
    Ok(chunks)
}

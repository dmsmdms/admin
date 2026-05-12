use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run(cancel_token: CancellationToken) {
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut terminate = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = interrupt.recv() => {
            info!("Received SIGINT");
        },
        _ = terminate.recv() => {
            info!("Received SIGTERM");
        },
    }

    cancel_token.cancel();
}

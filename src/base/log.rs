use chrono::Utc;
use serde::Serialize;
use std::fmt::Debug;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;
use tracing::{Event, Level, Subscriber, field::Field};
use tracing_subscriber::{
    EnvFilter, fmt,
    layer::{Context, Layer, SubscriberExt},
    util::SubscriberInitExt,
};

#[derive(Serialize)]
#[allow(dead_code)]
enum LogType {
    Error,
    Warning,
}

#[derive(Serialize)]
pub struct Log {
    log_type: LogType,
    ts_sec: i64,
    ts_msec: u32,
    file: String,
    msg: String,
}

#[allow(dead_code)]
struct CustomLayer {
    tx: Sender<Log>,
}

impl<S> Layer<S> for CustomLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let log_type = match *metadata.level() {
            Level::ERROR => LogType::Error,
            Level::WARN => LogType::Warning,
            _ => return,
        };

        let mut msg = String::new();
        event.record(&mut |field: &Field, value: &dyn Debug| {
            if field.name() == "message" {
                msg = format!("{value:?}");
            }
        });

        let now = Utc::now();
        let log = Log {
            log_type,
            ts_sec: now.timestamp(),
            ts_msec: now.timestamp_subsec_millis(),
            file: format!(
                "{}:{}",
                metadata.file().unwrap_or("unknown"),
                metadata.line().unwrap_or(0)
            ),
            msg,
        };
        self.tx.try_send(log).ok();
    }
}

pub fn init() -> Option<Receiver<Log>> {
    let fmt_layer = fmt::layer().with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry().with(fmt_layer).init();
    None
}

pub async fn run(_rx: Receiver<Log>, _cancel_token: CancellationToken) {
    // Remote log transmission disabled
}

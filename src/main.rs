use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber
                ::EnvFilter
                ::try_from_default_env()
                .unwrap_or_else(|_| {
                    "feedback=debug,tower_http=debug,axum::rejection=trace"
                        .into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    feedback::server::Server::run().await
}

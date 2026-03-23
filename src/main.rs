use std::sync::Arc;

use teloxide::prelude::*;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

use tg_downloader::{
    downloader::Downloader,
    error::ErrorSender,
    handler::{download_request, mk_round},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    tracing::info!("Initializing...");
    let bot = Arc::new(Bot::from_env());
    let downloader = Arc::new(Downloader::from_env().inspect_err(|e| tracing::error!(?e))?);
    let schema = Update::filter_message()
        .filter_map(|update: Update| update.from().cloned())
        .branch(Message::filter_text().endpoint(download_request))
        .branch(Message::filter_video().endpoint(mk_round));
    let error_sender = ErrorSender::with_bot(bot.clone());
    tracing::info!("Starting tiktok downloader bot...");
    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![downloader])
        .error_handler(error_sender)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    tracing::info!("Closing bot... Goodbye!");
    Ok(())
}

pub fn init_logging() {
    // This is an env var we made up to control the output log format.
    let output_type = std::env::var("LOG_OUTPUT_TYPE").unwrap_or("json".to_owned());

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // This won't scale well for more than two options
    let (pretty, json) = if output_type.to_lowercase().eq("pretty") {
        (None, Some(tracing_subscriber::fmt::layer().pretty()))
    } else {
        (Some(tracing_subscriber::fmt::layer().json()), None)
    };

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(json)
        .with(pretty)
        .init();
}

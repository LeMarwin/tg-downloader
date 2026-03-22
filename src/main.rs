use std::sync::Arc;

use handler::message_handler;
use teloxide::prelude::*;

mod downloader;
mod handler;
mod url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::info!("Starting tiktok downloader bot...");
    let bot = Bot::from_env();
    let matcher = Arc::new(url::UrlChecker::new()?);
    let schema = Update::filter_message()
        .filter_map(|update: Update| update.from().cloned())
        .branch(
            Message::filter_text().endpoint(move |bot, user, message_text| {
                message_handler(bot, user, message_text, matcher.clone())
            }),
        );
    Dispatcher::builder(bot, schema)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    log::info!("Closing bot... Goodbye!");
    Ok(())
}

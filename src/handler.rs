use std::sync::Arc;

use teloxide::{
    prelude::Requester,
    types::{InputFile, User},
    Bot,
};

use crate::{
    downloader::{download_audio_only, Downloader},
    url::UrlChecker,
};

pub async fn message_handler(
    bot: Bot,
    user: User,
    text: String,
    matcher: Arc<UrlChecker>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = user.username.clone().unwrap_or(user.full_name());
    let check_result = matcher.check(&text);
    let url_type = check_result
        .as_ref()
        .map_or_else(|| "Unrecognized".to_string(), |v| format!("{v:?}"));
    log::info!("[{name}][{url_type}]: {text}");

    let Some(url_type) = check_result else {
        bot.send_message(user.id, "Unrecognized url!").await?;
        return Ok(());
    };

    match Downloader::from_url_type(&text, url_type).download().await {
        Ok(fpath) => {
            if url_type.is_video() {
                bot.send_video(user.id, InputFile::file(&fpath)).await?;
                let _ = std::fs::remove_file(fpath);
            } else {
                bot.send_audio(user.id, InputFile::file(&fpath)).await?;
                let _ = std::fs::remove_file(fpath);
            }
        }
        Err(e) => {
            if matches!(url_type, crate::url::UrlType::YoutubeAudio) {
                log::info!("Here");
                match download_audio_only(&text).await {
                    Ok(fpath) => {
                        bot.send_audio(user.id, InputFile::file(&fpath)).await?;
                        let _ = std::fs::remove_file(fpath);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("{e}");
                        bot.send_message(user.id, format!("{e}")).await?;
                    }
                }
            } else {
                log::error!("{e}");
                bot.send_message(user.id, format!("{e}")).await?;
            }
        }
    }
    Ok(())
}

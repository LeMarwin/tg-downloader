//! Query callback parsing and handling

use std::{str::FromStr, sync::Arc};

use teloxide::{
    Bot,
    prelude::Requester as _,
    types::{CallbackQuery, ChatId, MaybeInaccessibleMessage},
};

use crate::{
    ErrorExt as _, HandlerResult, OptExt as _,
    downloader::{Downloader, Format},
    handler::{upload_audio, upload_video},
    url::UrlMatcher,
};

/// Error when parsing callback query
#[derive(Debug, thiserror::Error)]
#[expect(missing_docs)]
pub enum Error {
    #[error("Malformed query: {0}")]
    Malformed(String),
    #[error("Failed to parse id: {ty} {e}")]
    Parse {
        ty: String,
        e: <u32 as FromStr>::Err,
    },
}

/// Parsed callback data
pub enum Query {
    /// Selected format: audio
    FormatAudio {
        /// Format id
        id: u32,
    },
    /// Selected format: video
    FormatVideo {
        /// Format id
        id: u32,
    },
    /// Delete the keyboard message
    Close,
}

impl FromStr for Query {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let malformed = || Self::Err::Malformed(s.to_owned());
        let mut parts = s.split(':');
        let ty = parts.next().ok_or_else(malformed)?;
        if ty == "close" {
            return Ok(Self::Close);
        }
        let id = parts
            .next()
            .ok_or_else(malformed)?
            .parse()
            .map_err(|e| Self::Err::Parse {
                ty: ty.to_owned(),
                e,
            })?;
        let fmt = match ty {
            "audio" => Self::FormatAudio { id },
            "video" => Self::FormatVideo { id },
            _ => return Err(malformed()),
        };
        Ok(fmt)
    }
}

/// Handle callback query
pub async fn handle_callback_query(
    bot: Arc<Bot>,
    downloader: Arc<Downloader>,
    q: CallbackQuery,
) -> HandlerResult<()> {
    let CallbackQuery {
        id,
        from,
        message,
        data,
        ..
    } = q;

    bot.answer_callback_query(id.clone()).await?;
    let chat_id: ChatId = from.id.into();
    let query: Query = data.context("Query w/o data")?.parse().with_chat(chat_id)?;
    let format = match &query {
        Query::FormatAudio { id } => Format::Audio(*id),
        Query::FormatVideo { id } => Format::Video(*id),
        Query::Close => {
            if let Some(msg) = message {
                bot.delete_message(chat_id, msg.id()).await?;
            }
            return Ok(());
        }
    };

    let Some(url) = get_source_url(&message) else {
        if let Some(msg) = message {
            bot.delete_message(chat_id, msg.id()).await?;
        }
        return None::<()>.context_chat(chat_id, "Failed to find original url. Send again");
    };
    let path = downloader.download_with_format(&url, format).await?;
    tracing::info!(user=from.full_name(), %chat_id, url, path = %path.display());

    let res = if matches!(query, Query::FormatVideo { .. }) {
        upload_video(&bot, chat_id, &path).await
    } else {
        upload_audio(&bot, chat_id, &path).await
    };
    tokio::fs::remove_file(path).await?;
    res
}

fn get_source_url(source: &Option<MaybeInaccessibleMessage>) -> Option<String> {
    Some(
        UrlMatcher::get_match(
            source
                .as_ref()?
                .regular_message()?
                .reply_to_message()?
                .text()?,
        )?
        .0
        .to_owned(),
    )
}

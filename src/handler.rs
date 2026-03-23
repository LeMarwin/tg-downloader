//! Request handlers

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::Bytes;
use ez_ffmpeg::{Input, Output};
use teloxide::{
    Bot,
    net::Download as _,
    payloads::SendVideoSetters as _,
    prelude::Requester as _,
    types::{ChatId, FileId, InputFile, User, Video},
};
use tokio::io::AsyncReadExt as _;

use crate::{
    downloader::Downloader,
    error::{Error, ErrorExt as _, HandlerResult},
    url::{URL_CHECKER, UrlType},
    util::{self, VideoMeta},
};

/// Handle download requests
pub async fn download_request(
    bot: Arc<Bot>,
    user: User,
    text: String,
    downloader: Arc<Downloader>,
) -> HandlerResult<()> {
    let name = user.username.clone().unwrap_or(user.full_name());
    let chat_id: ChatId = user.id.into();
    let check_result = URL_CHECKER.check(&text);
    let url_type = check_result
        .ok_or(Error::UnrecognizedUrl(text.clone()))
        .with_chat(user.id.into())?;
    let url = if matches!(url_type, UrlType::YoutubeVideo) {
        text.strip_prefix("video ").unwrap_or(&text)
    } else {
        &text
    };
    tracing::info!(user=name, %chat_id, %url_type, url, "Request");

    let path = downloader
        .download(url, &url_type)
        .await
        .with_chat(chat_id)?;
    tracing::info!(user=name, %chat_id, %url_type, url, path = %path.display());
    let res = if url_type.is_video() {
        upload_video(&bot, chat_id, &path).await
    } else {
        upload_audio(&bot, chat_id, &path).await
    };
    tokio::fs::remove_file(&path).await?;
    res
}

const FFMPEG_FILTER: &str = r"
[0:v]crop=min(in_w\,in_h):min(in_w\,in_h)[main]; 
[main]scale=min(in_w\,640):min(in_h\,640)[main]; 
[1:v][main]scale2ref[mask][main]; 
[main][mask]overlay=(W-w)/2:(H-h)/2
";

/// Convert incoming video into a round video note
pub async fn mk_round(bot: Arc<Bot>, user: User, video: Video) -> HandlerResult<()> {
    let name = user.username.clone().unwrap_or(user.full_name());
    let cid: ChatId = user.id.into();
    tracing::info!(user = name, "Round request");
    let file = download_file(&bot, cid, video.file.id).await?;
    let byte_input = Input::new(file.path().to_string_lossy())
        .set_start_time_us(0)
        .set_stop_time_us(60_000_000);
    let mut output_file = async_tempfile::TempFile::new().await?;
    let byte_output = Output::new(output_file.file_path().to_string_lossy()).set_format("mp4");
    ez_ffmpeg::FfmpegContext::builder()
        .inputs(vec![byte_input, Input::new("overlay.png")])
        .filter_desc(FFMPEG_FILTER)
        .output(byte_output)
        .build()?
        .start()?
        .await?;
    let mut out = vec![];
    output_file.read_to_end(&mut out).await?;
    let f = InputFile::memory(Bytes::from_owner(out));
    bot.send_video_note(cid, f).await?;
    Ok(())
}

enum DownloadedFilePath {
    Local(PathBuf),
    Tmp(async_tempfile::TempFile),
}

impl DownloadedFilePath {
    fn path(&self) -> &Path {
        match self {
            Self::Local(path_buf) => path_buf,
            Self::Tmp(temp_file) => temp_file.file_path(),
        }
    }
}

/// 20Mb download size limit for non-local API calls
const SIZE_LIMIT: u32 = 20 * 1024 * 1024;

async fn download_file(
    bot: &Bot,
    chat_id: ChatId,
    file_id: FileId,
) -> HandlerResult<DownloadedFilePath> {
    let file = bot.get_file(file_id).await.with_chat(chat_id)?;
    // If `TELOXIDE_API_URL` is set, assume local API server, thus the filepath is local, no extra steps required
    if std::env::var("TELOXIDE_API_URL").is_ok() {
        Ok(DownloadedFilePath::Local(PathBuf::from(file.path)))
    } else if file.size > SIZE_LIMIT {
        Err(Error::FileTooLarge(file.size)).with_chat(chat_id)
    } else {
        let mut buf = async_tempfile::TempFile::new().await?;
        bot.download_file(&file.path, &mut buf)
            .await
            .with_chat(chat_id)?;
        Ok(DownloadedFilePath::Tmp(buf))
    }
}

async fn upload_video(bot: &Bot, chat_id: ChatId, input: &Path) -> HandlerResult<()> {
    let VideoMeta {
        width,
        height,
        duration_sec,
        thumbnail,
    } = util::video_meta(input).await?;
    bot.send_video(chat_id, InputFile::file(input))
        .duration(duration_sec)
        .supports_streaming(true)
        .thumbnail(InputFile::file(thumbnail.file_path()))
        .width(width)
        .height(height)
        .await
        .map_err(Error::RequestError)?;
    Ok(())
}

async fn upload_audio(bot: &Bot, chat_id: ChatId, input: &Path) -> HandlerResult<()> {
    bot.send_audio(chat_id, InputFile::file(input))
        .await
        .map_err(Error::RequestError)?;
    Ok(())
}

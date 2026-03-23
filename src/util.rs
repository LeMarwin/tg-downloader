use std::{
    env,
    path::{Path, PathBuf},
};

use async_tempfile::TempFile;
use ffprobe::ffprobe_config;

pub struct VideoMeta {
    pub width: u32,
    pub height: u32,
    pub duration_sec: u32,
    pub thumbnail: TempFile,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("FFprobe: {0}")]
    Fprobe(ffprobe::FfProbeError),
    #[error("File contains no streams: {0}")]
    NoStream(PathBuf),
    #[error("FFMpeg: {0}")]
    Ffmpeg(#[from] ez_ffmpeg::error::Error),
    #[error("TMPFile: {0}")]
    TmpFile(#[from] async_tempfile::Error),
    #[error("FFMPEG_PATH: {0}")]
    FfmpegPath(std::env::VarError),
    #[error("Unknown width")]
    Width,
    #[error("Width 0")]
    WidthZero,
    #[error("Unknown height")]
    Height,
    #[error("Height 0")]
    HeightZero,
}

/// Get metadata for a file
pub async fn video_meta(input: &Path) -> Result<VideoMeta, Error> {
    let ffmpeg_path = env::var("FFMPEG_PATH").map_err(Error::FfmpegPath)?;
    tracing::info!(ffmpeg_path);
    let ffprobe_path = format!("{ffmpeg_path}/ffprobe");
    let meta = ffprobe_config(
        ffprobe::ConfigBuilder::new()
            .ffprobe_bin(ffprobe_path)
            .build(),
        &input,
    )
    .map_err(Error::Fprobe)?;
    let stream = meta
        .streams
        .into_iter()
        .find(|s| s.codec_type == Some("video".to_owned()))
        .ok_or(Error::NoStream(input.to_path_buf()))?;
    let width = stream.width.ok_or(Error::Width)? as u32;
    let height = stream.width.ok_or(Error::Height)? as u32;
    let duration_sec = stream
        .duration
        .as_ref()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or_default()
        .ceil() as u32;
    if width == 0 {
        return Err(Error::WidthZero);
    }
    if height == 0 {
        return Err(Error::HeightZero);
    }
    let filter = if width > height {
        "scale=320:-1"
    } else {
        "scale=-1:320"
    };
    let thumbnail = async_tempfile::TempFile::new().await?;
    ez_ffmpeg::FfmpegContext::builder()
        .input(ez_ffmpeg::Input::new(input.to_string_lossy()))
        .filter_desc(filter)
        .output(
            ez_ffmpeg::Output::new(thumbnail.file_path().to_string_lossy())
                .set_max_video_frames(1)
                .set_format("jpg"),
        )
        .build()?
        .start()?
        .await?;

    Ok(VideoMeta {
        width,
        height,
        thumbnail,
        duration_sec,
    })
}

//! Utility functions

use std::{
    env,
    path::{Path, PathBuf},
};

use async_tempfile::TempFile;
use ffprobe::ffprobe_config;

/// Video metadata
pub struct VideoMeta {
    /// Video width
    pub width: u32,
    /// Video height
    pub height: u32,
    /// Duration in seconds
    pub duration_sec: u32,
    /// Handle to thumbnail file.
    /// File is removed when the handle is dropped
    pub thumbnail: TempFile,
}

#[expect(missing_docs)]
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
    FfmpegPath(env::VarError),
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
        input,
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
    let thumbnail = TempFile::new().await?;
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

/// Format the size with SI suffixes.
/// Left-pads with spaces to a same-length string
#[expect(clippy::arithmetic_side_effects)]
pub fn fmt_size(size: u64) -> String {
    use f128::f128;
    use num_traits::real::Real as _;

    const SUFFIX: [&str; 9] = ["B ", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let unit: f128 = f128::from(1024.0);
    let size = f128::from(size);

    if size < unit {
        return format!("{size:>7} B ");
    }

    let base = size.log10() / unit.log10();
    let size: f64 = unit.powf(base - base.floor()).into();
    let base: f64 = base.into();
    let result = format!("{size:.2}").trim_end_matches(".0").to_owned();
    format!("{result} {}", SUFFIX[base.floor() as usize])
}

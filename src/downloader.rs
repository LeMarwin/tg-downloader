//! Download helper

use std::path::PathBuf;

use tokio::process::Command;

use crate::{error::Error, url::UrlType};

/// Download helper
pub struct Downloader {
    yt_dlp: String,
    ffmpeg: String,
}

#[expect(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("YT_DLP_PATH: {0}")]
    Ytdlp(std::env::VarError),
    #[error("FFMPEG_PATH: {0}")]
    Ffmpeg(std::env::VarError),
}

impl Downloader {
    /// Initialize a downloader from env vars
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            yt_dlp: std::env::var("YT_DLP_PATH").map_err(EnvError::Ytdlp)?,
            ffmpeg: std::env::var("FFMPEG_PATH").map_err(EnvError::Ffmpeg)?,
        })
    }

    /// Download a url with a given type
    pub async fn download(&self, url: &str, url_type: &UrlType) -> Result<PathBuf, Error> {
        let mut cmd = Command::new(&self.yt_dlp);
        cmd.args([
            "--no-progress",
            "--print",
            "after_move:filepath",
            "-P",
            "./stored/",
            "-o",
            "%(title)s.%(ext)s",
            "--restrict-filenames",
            &format!("--ffmpeg-location={}", self.ffmpeg),
            "-t",
            url_type.yt_dlp_format(),
            url,
        ]);
        cmd.stdout(std::process::Stdio::piped());
        let res = cmd.spawn()?.wait_with_output().await?;
        let path_str = String::from_utf8(res.stdout).map_err(std::io::Error::other)?;
        Ok(PathBuf::from(path_str.trim()))
    }
}

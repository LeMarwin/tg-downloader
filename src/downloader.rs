use std::path::PathBuf;

use tokio::process::Command;

use crate::{error::Error, url::UrlType};

pub struct Downloader {
    yt_dlp: String,
    ffmpeg: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("YT_DLP_PATH: {0}")]
    Ytdlp(std::env::VarError),
    #[error("FFMPEG_PATH: {0}")]
    Ffmpeg(std::env::VarError),
}

impl Downloader {
    pub fn from_env() -> Result<Self, EnvError> {
        Ok(Self {
            yt_dlp: std::env::var("YT_DLP_PATH").map_err(EnvError::Ytdlp)?,
            ffmpeg: std::env::var("FFMPEG_PATH").map_err(EnvError::Ffmpeg)?,
        })
    }

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
        let path_str = String::from_utf8(res.stdout).map_err(|e| std::io::Error::other(e))?;
        Ok(PathBuf::from(path_str.trim()))
    }
}

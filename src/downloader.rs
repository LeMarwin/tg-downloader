//! Download helper

use std::path::PathBuf;

use itertools::Either;
use tokio::process::Command;

use crate::{dlp_info::DlpInfo, error::Error, url::UrlType};

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

    /// Get json info for an url
    pub async fn get_info_json(&self, url: &str) -> Result<DlpInfo, Error> {
        let mut cmd = Command::new(&self.yt_dlp);
        cmd.args(["-j", "--no-download", url]);
        cmd.stdout(std::process::Stdio::piped());
        let res = cmd.spawn()?.wait_with_output().await?;
        let info = serde_json::from_slice(&res.stdout)?;
        Ok(info)
    }

    /// Download a url using the stored json file
    pub async fn download_with_format(&self, url: &str, format: Format) -> Result<PathBuf, Error> {
        let mut cmd = Command::new(&self.yt_dlp);
        let ffmpeg = format!("--ffmpeg-location={}", self.ffmpeg);
        let args = [
            "--no-progress",
            "--print",
            "after_move:filepath",
            "-P",
            "./stored/",
            "-o",
            "%(title)s.%(ext)s",
            &ffmpeg,
        ]
        .into_iter()
        .map(ToOwned::to_owned)
        .chain(format.args())
        .chain(std::iter::once(url.to_owned()));
        cmd.args(args);
        cmd.stdout(std::process::Stdio::piped());
        let res = cmd.spawn()?.wait_with_output().await?;
        let path_str = String::from_utf8(res.stdout).map_err(std::io::Error::other)?;
        Ok(PathBuf::from(path_str.trim()))
    }
}

/// Format info for the downloader
#[derive(Debug, Clone, Copy)]
pub enum Format {
    /// Audio
    Audio(u32),
    /// Video
    Video(u32),
}

impl Format {
    fn args(self) -> impl IntoIterator<Item = String> + use<> {
        match self {
            Self::Audio(id) => Either::Left(
                ["-f", &id.to_string(), "-x", "--audio-format", "mp3"]
                    .map(ToOwned::to_owned)
                    .into_iter(),
            ),
            Self::Video(id) => Either::Right(
                [
                    "-f",
                    &format!("{id}+ba"),
                    "--merge-output-format",
                    "mp4",
                    "--remux-video",
                    "mp4",
                ]
                .map(ToOwned::to_owned)
                .into_iter(),
            ),
        }
    }
}

use std::path::PathBuf;

use tokio::process::Command;

use crate::url::UrlType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Command execution: {0}")]
    Command(#[from] std::io::Error),
    #[error("yt-dlp output is not utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Destination for downloaded file not found: {0}")]
    Destination(String),
    #[error("Invalid path")]
    InvalidPath,
}

/// Downloader info
pub struct Downloader {
    url: String,
    url_type: UrlType,
}

impl Downloader {
    pub fn from_url_type(url: &str, url_type: UrlType) -> Self {
        // youtube video matches on "video https://..." or "https://... video"
        let url = match url_type {
            UrlType::YoutubeAudio | UrlType::Tiktok | UrlType::InstaReel => url.to_string(),
            UrlType::YoutubeVideo => url
                .strip_prefix("video ")
                .unwrap_or(url.strip_suffix(" video").unwrap_or(url))
                .to_string(),
            UrlType::Webm => url.lines().last().unwrap_or(url).to_string(),
        };
        Self { url, url_type }
    }

    pub async fn download(self) -> Result<String, Error> {
        let format = match self.url_type {
            UrlType::YoutubeAudio => "ba[ext=m4a]",
            UrlType::YoutubeVideo => "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4] / bv*+ba/b",
            UrlType::Tiktok | UrlType::Webm | UrlType::InstaReel => "bv*+ba/b",
        };
        let out = Command::new("yt-dlp")
            .args([
                "-P",
                "./stored/",
                "-o",
                "%(title)s.%(ext)s",
                "-f",
                format,
                &self.url,
            ])
            .output()
            .await
            .map_err(Error::Command)?
            .stdout;
        let output = String::from_utf8(out)?;
        let path = path_from_ytdlp_output(&output).ok_or(Error::Destination(output))?;
        if matches!(self.url_type, UrlType::Webm) {
            convert_to_mp4(path).await
        } else {
            if matches!(self.url_type, UrlType::InstaReel) {
                repack_reel(&path).await?;
            }
            Ok(path)
        }
    }
}

pub async fn download_audio_only(url: &str) -> Result<String, Error> {
    let out = Command::new("yt-dlp")
        .args([
            "-P",
            "./stored/",
            "-o",
            "%(title)s.%(ext)s",
            "-f",
            "ba",
            url,
        ])
        .output()
        .await
        .map_err(Error::Command)?
        .stdout;
    let output = String::from_utf8(out)?;
    println!("{output}");
    let path = path_from_ytdlp_output(&output).ok_or(Error::Destination(output))?;
    extract_m4a_audio(path).await
}

fn path_from_ytdlp_output(output: &str) -> Option<String> {
    let mut lines = output.lines();
    let merger = lines
        .find(|s| s.contains("Merger"))
        .and_then(|s| s.split("\"").nth(1).map(String::from));
    if let Some(path) = merger {
        Some(path)
    } else {
        let mut lines = output.lines();
        lines
            .find(|s| s.contains("Destination:"))
            .and_then(|l| l.split("Destination: ").last())
            .map(|l| l.trim().to_string())
    }
}

async fn extract_m4a_audio(path: String) -> Result<String, Error> {
    let mut pbuf = PathBuf::from(&path);
    if pbuf.extension().and_then(|e| e.to_str()) == Some("m4a") {
        log::info!("Here?");
        return Ok(path);
    }

    if !pbuf.set_extension("m4a") {
        return Err(Error::InvalidPath);
    };

    let outfile = pbuf.to_string_lossy().to_string();
    Command::new("ffmpeg")
        .args(["-i", &path, "-vn", "-c:a", "copy", &outfile])
        .output()
        .await?;
    let _ = std::fs::remove_file(path);
    Ok(outfile)
}

async fn convert_to_mp4(path: String) -> Result<String, Error> {
    let mut pbuf = PathBuf::from(&path);
    if pbuf.extension().and_then(|e| e.to_str()) == Some("mp4") {
        return Ok(path);
    }

    if !pbuf.set_extension("mp4") {
        return Err(Error::InvalidPath);
    };

    let outfile = pbuf.to_string_lossy().to_string();

    Command::new("ffmpeg")
        .args(["-i", &path, &outfile])
        .output()
        .await?;

    let _ = std::fs::remove_file(path);
    Ok(outfile)
}

async fn repack_reel(path: &str) -> Result<(), Error> {
    let pbuf = PathBuf::from(path);
    if let Some(fname) = pbuf.file_name() {
        let tmp = pbuf
            .with_file_name(format!("tmp_{}", fname.to_string_lossy()))
            .to_string_lossy()
            .into_owned();
        Command::new("mv").args([path, &tmp]).output().await?;
        Command::new("ffmpeg")
            .args([
                "-i",
                &tmp,
                "-c:v",
                "libx265",
                "-preset",
                "ultrafast",
                "-c:a",
                "copy",
                path,
            ])
            .output()
            .await?;
        let _ = std::fs::remove_file(tmp);
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::path_from_ytdlp_output;

    #[test]
    fn test_path_from_merger() {
        let output = r#"[youtube] Extracting URL: https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS
            [youtube] ytWz0qVvBZ0: Downloading webpage
            [youtube] ytWz0qVvBZ0: Downloading tv client config
            [youtube] ytWz0qVvBZ0: Downloading player 73381ccc-main
            [youtube] ytWz0qVvBZ0: Downloading tv player API JSON
            [youtube] ytWz0qVvBZ0: Downloading ios player API JSON
            [youtube] ytWz0qVvBZ0: Downloading m3u8 information
            [info] ytWz0qVvBZ0: Downloading 1 format(s): 399+140
            [download] Destination: ./stored/♪ Diggy Diggy Hole.f399.mp4
            [download] 100% of    8.78MiB in 00:00:01 at 5.60MiB/s
            [download] Destination: ./stored/♪ Diggy Diggy Hole.f140.m4a
            [download] 100% of    3.83MiB in 00:00:00 at 5.42MiB/s
            [Merger] Merging formats into "./stored/♪ Diggy Diggy Hole.mp4"
            Deleting original file ./stored/♪ Diggy Diggy Hole.f399.mp4 (pass -k to keep)
            Deleting original file ./stored/♪ Diggy Diggy Hole.f140.m4a (pass -k to keep)
            "#;
        let path = path_from_ytdlp_output(output);

        assert_eq!(path, Some("./stored/♪ Diggy Diggy Hole.mp4".to_string()))
    }

    #[test]
    fn test_path_from_destination() {
        let output = r#"
            [vm.tiktok] Extracting URL: https://vt.tiktok.com/ZSFhj2JFc/
            [vm.tiktok] ZSFhj2JFc: Downloading webpage
            [TikTok] Extracting URL: https://www.tiktok.com/@/video/7334135375243250986?_r=1&_d=secCgYIASAHKAESPgo8CQBN93r1ED5YjSAQbkX...ck&share_app_id=1233
            [TikTok] 7334135375243250986: Downloading webpage
            [info] 7334135375243250986: Downloading 1 format(s): bytevc1_1080p_1475820-1
            [download] Destination: ./stored/What did he say？ [7334135375243250986].mp4
            [download] 100% of    2.35MiB in 00:00:00 at 3.25MiB/s

        "#;

        let path = path_from_ytdlp_output(output);

        assert_eq!(
            path,
            Some("./stored/What did he say？ [7334135375243250986].mp4".to_string())
        )
    }
}

use std::{collections::HashMap, iter::FromIterator, sync::LazyLock};

use lazy_regex::{regex, Lazy};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlType {
    YoutubeAudio,
    YoutubeVideo,
    InstaReel,
    Tiktok,
    Webm,
}

impl std::fmt::Display for UrlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlType::YoutubeAudio => f.write_str("Youtube audio"),
            UrlType::YoutubeVideo => f.write_str("Youtube video"),
            UrlType::InstaReel => f.write_str("Instagram reel"),
            UrlType::Tiktok => f.write_str("Tiktok"),
            UrlType::Webm => f.write_str("webm"),
        }
    }
}

impl UrlType {
    pub fn is_video(&self) -> bool {
        match self {
            UrlType::YoutubeAudio => false,
            UrlType::YoutubeVideo => true,
            UrlType::InstaReel => true,
            UrlType::Tiktok => true,
            UrlType::Webm => true,
        }
    }

    pub fn yt_dlp_format(&self) -> &'static str {
        match self {
            UrlType::YoutubeAudio => "mp3",
            UrlType::YoutubeVideo => "mp4",
            UrlType::InstaReel => "mp4",
            UrlType::Tiktok => "mp4",
            UrlType::Webm => "mp4",
        }
    }
}

pub struct UrlChecker(HashMap<UrlType, &'static Lazy<Regex>>);

pub static URL_CHECKER: LazyLock<UrlChecker> = LazyLock::new(|| UrlChecker::new());

impl UrlChecker {
    pub fn new() -> Self {
        Self(HashMap::from_iter([
            (
                UrlType::YoutubeAudio,
                regex!(
                    r"^((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?$"
                ),
            ),
            (
                UrlType::YoutubeVideo,
                regex!(
                    r#"^((video )((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?$)|(((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?( video)$)"#
                ),
            ),
            (
                UrlType::Tiktok,
                regex!(
                    r#"https://(vt\.|m\.|www\.|)tiktok\.com/((@[a-zA-Z0-9_]*/video/[a-zA-Z0-9_\?=&]*)|([a-zA-Z0-9_]*))"#
                ),
            ),
            (UrlType::Webm, regex!(r#"(?:http)(?:.+/)(.+)(.webm)$"#)),
            (
                UrlType::InstaReel,
                regex!(r#"^((?:https?:)?//)?((?:www|m)\.)?((?:instagram\.com)/reel/)"#),
            ),
        ]))
    }

    pub fn check(&self, input: &str) -> Option<UrlType> {
        self.0
            .iter()
            .find_map(|(k, v)| v.is_match(input).then_some(*k))
    }
}

#[cfg(test)]
mod test {
    use crate::url::UrlType;

    use super::UrlChecker;

    #[test]
    fn match_tiktok() {
        let matcher = UrlChecker::new();

        let r = matcher.check("https://vt.tiktok.com/ZSFhj2JFc/");
        assert_eq!(r, Some(UrlType::Tiktok));
    }

    #[test]
    fn match_youtube_audio() {
        let matcher = UrlChecker::new();

        let r = matcher.check("https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS");
        assert_eq!(r, Some(UrlType::YoutubeAudio));
    }

    #[test]
    fn match_youtube_video() {
        let matcher = UrlChecker::new();

        let r = matcher.check("video https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS");
        assert_eq!(r, Some(UrlType::YoutubeVideo));

        let r = matcher.check("https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS video");
        assert_eq!(r, Some(UrlType::YoutubeVideo));
    }
}

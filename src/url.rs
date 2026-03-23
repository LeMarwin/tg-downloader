//! URL matcher

use std::{collections::HashMap, iter::FromIterator as _, sync::LazyLock};

use lazy_regex::{Lazy, regex};
use regex::Regex;

/// Supported url types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlType {
    /// Youtube audio
    YoutubeAudio,
    /// Youtube video
    YoutubeVideo,
    /// Instagram reel
    InstaReel,
    /// Tiktok
    Tiktok,
    /// Webm
    Webm,
}

impl std::fmt::Display for UrlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::YoutubeAudio => f.write_str("Youtube audio"),
            Self::YoutubeVideo => f.write_str("Youtube video"),
            Self::InstaReel => f.write_str("Instagram reel"),
            Self::Tiktok => f.write_str("Tiktok"),
            Self::Webm => f.write_str("webm"),
        }
    }
}

impl UrlType {
    /// Check if the [`UrlType`] is a video
    pub const fn is_video(&self) -> bool {
        match self {
            Self::YoutubeAudio => false,
            Self::YoutubeVideo | Self::InstaReel | Self::Tiktok | Self::Webm => true,
        }
    }

    /// Argument for `yt-dlp` `-t`
    pub const fn yt_dlp_format(&self) -> &'static str {
        match self {
            Self::YoutubeAudio => "mp3",
            Self::YoutubeVideo | Self::InstaReel | Self::Tiktok | Self::Webm => "mp4",
        }
    }
}

/// Url checker helper
pub struct UrlChecker(HashMap<UrlType, &'static Lazy<Regex>>);

/// Globally accessible checker
pub static URL_CHECKER: LazyLock<UrlChecker> = LazyLock::new(UrlChecker::default);

impl UrlChecker {
    /// Check url type
    pub fn check(&self, input: &str) -> Option<UrlType> {
        self.0
            .iter()
            .find_map(|(k, v)| v.is_match(input).then_some(*k))
    }
}

impl Default for UrlChecker {
    fn default() -> Self {
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
}

#[cfg(test)]
mod test {
    use crate::url::{URL_CHECKER, UrlType};

    #[test]
    fn match_tiktok() {
        let r = URL_CHECKER.check("https://vt.tiktok.com/ZSFhj2JFc/");
        assert_eq!(r, Some(UrlType::Tiktok));
    }

    #[test]
    fn match_youtube_audio() {
        let r = URL_CHECKER.check("https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS");
        assert_eq!(r, Some(UrlType::YoutubeAudio));
    }

    #[test]
    fn match_youtube_video() {
        let r = URL_CHECKER.check("video https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS");
        assert_eq!(r, Some(UrlType::YoutubeVideo));

        let r = URL_CHECKER.check("https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS video");
        assert_eq!(r, Some(UrlType::YoutubeVideo));
    }
}

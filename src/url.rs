//! URL matcher

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

static YOUTUBE: &Lazy<Regex> = regex!(
    r"((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?"
);

static TIKTOK: &Lazy<Regex> = regex!(
    r#"https://(vt\.|m\.|www\.|)tiktok\.com/((@[a-zA-Z0-9_]*/video/[a-zA-Z0-9_\?=&]*)|([a-zA-Z0-9_]*))"#
);
static WEBM: &Lazy<Regex> = regex!(r#"(?:http)(?:.+/)(.+)(.webm)$"#);
static INSTA_REEL: &Lazy<Regex> =
    regex!(r#"((?:https?:)?//)?((?:www|m)\.)?((?:instagram\.com)/reel/)"#);

static AUDIO: &Lazy<Regex> = regex!(r"(^[aA]\w*\s)|(\s[aA]\w*$)");
static VIDEO: &Lazy<Regex> = regex!(r"(^(v\w*\s)|(V\w*\s))|((\sv\w*|\sV\w*)$)");

/// Url checker helper
pub struct UrlMatcher {}

impl UrlMatcher {
    /// Get [`UrlType`] match
    pub fn get_match(url: &str) -> Option<(&str, UrlType)> {
        let youtube_match = Self::match_youtube(url);
        if youtube_match.is_some() {
            return youtube_match;
        }
        [
            (UrlType::Webm, WEBM),
            (UrlType::Tiktok, TIKTOK),
            (UrlType::InstaReel, INSTA_REEL),
        ]
        .into_iter()
        .find_map(|(t, r)| r.is_match(url).then_some((url, t)))
    }

    fn match_youtube(url: &str) -> Option<(&str, UrlType)> {
        if let Some(youtube_url) = YOUTUBE.captures(url).and_then(|m| m.get(0)) {
            let bare_url = youtube_url.as_str();
            let ty = if VIDEO.is_match(url) | (url.contains("/shorts/") && !AUDIO.is_match(url)) {
                UrlType::YoutubeVideo
            } else {
                UrlType::YoutubeAudio
            };
            Some((bare_url, ty))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::url::{UrlMatcher, UrlType};

    #[test]
    fn test_url_matcher() {
        let short = "https://youtube.com/shorts/2Xn2QxECrek?si=czHvJBoblApUjMEp".to_owned();
        let regular = "https://www.youtube.com/watch?v=3B4524ot5BM".to_owned();
        let tiktok = "https://vt.tiktok.com/ZSFhj2JFc/".to_owned();
        let webm = "https://i.4cdn.org/wsg/1774070495471223.webm".to_owned();
        let insta = "https://www.instagram.com/reel/DAAAAAAAAAA".to_owned();
        let test_urls = [
            (UrlType::YoutubeVideo, short.clone(), short.clone()),
            (UrlType::YoutubeAudio, regular.clone(), regular.clone()),
            (UrlType::Tiktok, tiktok.clone(), tiktok),
            (UrlType::Webm, webm.clone(), webm),
            (UrlType::InstaReel, insta.clone(), insta),
        ]
        .into_iter()
        .chain(
            [
                (format!("audio {short}"), short.clone()),
                (format!("au {short}"), short.clone()),
                (format!("a {short}"), short.clone()),
                (format!("{short} a"), short.clone()),
                (format!("{short} audio"), short.clone()),
                (format!("{short} au"), short.clone()),
                (format!("Audio {short}"), short.clone()),
                (format!("Au {short}"), short.clone()),
                (format!("A {short}"), short.clone()),
                (format!("{short} A"), short.clone()),
                (format!("{short} Audio"), short.clone()),
                (format!("{short} Au"), short),
            ]
            .into_iter()
            .map(|(u, bare_url)| (UrlType::YoutubeAudio, u, bare_url)),
        )
        .chain(
            [
                (format!("video {regular}"), regular.clone()),
                (format!("vid {regular}"), regular.clone()),
                (format!("v {regular}"), regular.clone()),
                (format!("{regular} v"), regular.clone()),
                (format!("{regular} video"), regular.clone()),
                (format!("{regular} vid"), regular.clone()),
                (format!("Video {regular}"), regular.clone()),
                (format!("V {regular}"), regular.clone()),
                (format!("{regular} V"), regular.clone()),
                (format!("{regular} Video"), regular.clone()),
                (format!("{regular} Vid"), regular),
            ]
            .map(|(u, bare_url)| (UrlType::YoutubeVideo, u, bare_url)),
        );

        for (t, url, bare) in test_urls {
            assert_eq!(
                UrlMatcher::get_match(&url),
                Some((bare.as_str(), t)),
                "{url}"
            );
        }

        assert_eq!(UrlMatcher::get_match("Nonsense"), None);
        assert_eq!(UrlMatcher::get_match("https://www.google.com/"), None);
    }
}

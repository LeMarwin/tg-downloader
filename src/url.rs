use std::{collections::HashMap, iter::FromIterator};

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlType {
    YoutubeAudio,
    YoutubeVideo,
    InstaReel,
    Tiktok,
    Webm,
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
}

pub struct UrlChecker(HashMap<UrlType, Regex>);

impl UrlChecker {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self(HashMap::from_iter([
            (
                UrlType::YoutubeAudio,
                Regex::new(
                    r#"^((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?$"#,
                )?,
            ),
            (
                UrlType::YoutubeVideo,
                Regex::new(
                    r#"^((video )((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?$)|(((?:https?:)?//)?((?:www|m)\.)?((?:youtube\.com|youtu.be))(/(?:[\w\-]+\?v=|embed/|v/)?)([\w\-]+)(\S+)?( video)$)"#,
                )?,
            ),
            (
                UrlType::Tiktok,
                Regex::new(
                    r#"https://(vt\.|m\.|www\.|)tiktok\.com/((@[a-zA-Z0-9_]*/video/[a-zA-Z0-9_\?=&]*)|([a-zA-Z0-9_]*))"#,
                )?,
            ),
            (UrlType::Webm, Regex::new(r#"(?:http)(?:.+/)(.+)(.webm)$"#)?),
            (
                UrlType::InstaReel,
                Regex::new(r#"^((?:https?:)?//)?((?:www|m)\.)?((?:instagram\.com)/reel/)"#)?,
            ),
        ])))
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
        let matcher = UrlChecker::new().expect("");

        let r = matcher.check("https://vt.tiktok.com/ZSFhj2JFc/");
        assert_eq!(r, Some(UrlType::Tiktok));
    }

    #[test]
    fn match_youtube_audio() {
        let matcher = UrlChecker::new().expect("");

        let r = matcher.check("https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS");
        assert_eq!(r, Some(UrlType::YoutubeAudio));
    }

    #[test]
    fn match_youtube_video() {
        let matcher = UrlChecker::new().expect("");

        let r = matcher.check("video https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS");
        assert_eq!(r, Some(UrlType::YoutubeVideo));

        let r = matcher.check("https://youtu.be/ytWz0qVvBZ0?si=XVs8rAM2bx-9FEiS video");
        assert_eq!(r, Some(UrlType::YoutubeVideo));
    }
}

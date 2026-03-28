//! Helper to get download info from `yt-dlp`

use serde::Deserialize;

/// Download info from `yt-dlp`
#[derive(Debug, Deserialize)]
pub struct DlpInfo {
    /// Download id
    pub id: String,
    /// List of available formats
    pub formats: Vec<FormatInfo>,
}

/// Format
#[derive(Debug, Deserialize)]
#[serde(from = "FormatInfoRaw")]
pub enum FormatInfo {
    /// Catchall raw format
    Unknown(FormatInfoRaw),
    /// Audio
    Audio(Audio),
    /// Video
    Video(Video),
}

impl From<FormatInfoRaw> for FormatInfo {
    fn from(value: FormatInfoRaw) -> Self {
        let Ok(id) = value.format_id.parse() else {
            return Self::Unknown(value);
        };
        match (value.acodec.as_str(), value.vcodec.as_str()) {
            ("none", "none") => Self::Unknown(value),
            ("none", _) => Self::Video(Video {
                id,
                width: value.width.unwrap_or_default(),
                height: value.height.unwrap_or_default(),
                note: value.format_note,
                size: value.filesize_approx.unwrap_or_default(),
                bitrate: value.vbr.unwrap_or_default(),
            }),
            (_, "none") => Self::Audio(Audio {
                id,
                note: value.format_note,
                bitrate: value.vbr.unwrap_or_default(),
                size: value.filesize_approx.unwrap_or_default(),
            }),
            _ => Self::Unknown(value),
        }
    }
}

/// Raw format info for deserialization
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct FormatInfoRaw {
    format_note: String,
    format_id: String,
    resolution: String,
    width: Option<u32>,
    height: Option<u32>,
    filesize_approx: Option<u64>,
    abr: Option<f64>,
    vbr: Option<f64>,
    acodec: String,
    vcodec: String,
}

/// Audio
#[derive(Debug, PartialEq)]
pub struct Audio {
    /// Format id
    pub id: u32,
    /// Audio bitrate
    pub bitrate: f64,
    /// File size
    pub size: u64,
    /// Format note
    pub note: String,
}

impl PartialOrd for Audio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.bitrate.partial_cmp(&other.bitrate) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.size.partial_cmp(&other.size) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        None
    }
}

/// Video
#[derive(Debug)]
pub struct Video {
    /// Format id
    pub id: u32,
    /// Format note
    pub note: String,
    /// Video width
    pub width: u32,
    /// Video height
    pub height: u32,
    /// Approximate video size
    pub size: u64,
    /// Video bitrate
    pub bitrate: f64,
}

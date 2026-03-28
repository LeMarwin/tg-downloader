use std::collections::HashMap;

use itertools::Itertools as _;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{
    dlp_info::{Audio, FormatInfo, Video},
    util::fmt_size,
};

pub fn info_to_keyboard(formats: Vec<FormatInfo>) -> InlineKeyboardMarkup {
    let mut inline_keyboard = vec![];
    let (audio, videos) = process_formats(formats);
    for els in videos.chunks(2) {
        inline_keyboard.push(
            els.iter()
                .map(|v| {
                    let name = format!("{} ({})", v.note, fmt_size(v.size));
                    let data = format!("video:{}", v.id);
                    InlineKeyboardButton::callback(name, data)
                })
                .collect(),
        );
    }
    if let Some(audio) = audio {
        let name = format!("Audio ({})", fmt_size(audio.size));
        let data = format!("audio:{}", audio.id);
        inline_keyboard.push(vec![InlineKeyboardButton::callback(name, data)]);
    }
    inline_keyboard.push(vec![InlineKeyboardButton::callback("Close", "close:")]);
    InlineKeyboardMarkup { inline_keyboard }
}

fn process_formats(formats: Vec<FormatInfo>) -> (Option<Audio>, Vec<Video>) {
    let mut audio_best = None;
    let mut video_buckets: HashMap<String, Vec<Video>> = HashMap::new();

    for f in formats {
        match f {
            FormatInfo::Unknown(_) => continue,
            FormatInfo::Audio(audio) => {
                let current_best = audio_best.take();
                audio_best = if current_best.as_ref().is_some_and(|cb| *cb > audio) {
                    current_best
                } else {
                    Some(audio)
                };
            }
            FormatInfo::Video(video) => {
                video_buckets
                    .entry(video.note.clone())
                    .or_default()
                    .push(video);
            }
        }
    }

    let mut videos = video_buckets
        .into_values()
        .filter_map(|mut bucket| {
            bucket.sort_by_key(|v| v.size);
            bucket.into_iter().next_back()
        })
        .collect_vec();
    videos.sort_by_key(|v| v.size);
    (audio_best, videos)
}

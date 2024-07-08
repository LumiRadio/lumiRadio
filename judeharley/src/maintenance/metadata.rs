use std::path::Path;

use ffmpeg_next::DictionaryRef;
use lazy_static::lazy_static;

pub struct MusicMetadata {
    pub duration: f64,
    pub bitrate: i64,
    pub tags: Tags,
}

impl MusicMetadata {
    pub fn new<P: AsRef<Path>>(path: &P) -> std::io::Result<Self> {
        let format_ctx = ffmpeg_next::format::input(path)?;

        let file_size = std::fs::metadata(path)?.len();

        let duration = if format_ctx.duration() >= 0 {
            format_ctx.duration() as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64
        } else {
            0_f64
        };

        let bitrate = if format_ctx.bit_rate() >= 0 {
            format_ctx.bit_rate()
        } else if duration > 0_f64 {
            (file_size * 8) as i64 / duration as i64
        } else {
            0_i64
        };

        let tags = format_ctx.metadata().to_tags();
        tracing::debug!("Tags: {:?}", tags);

        Ok(Self {
            duration,
            bitrate,
            tags,
        })
    }
}

pub type Tags = Vec<(String, String)>;
pub trait ToTags {
    fn to_tags(&self) -> Tags;
}

fn tag_is_boring(key: &str) -> bool {
    lazy_static! {
        static ref BORING_PATTERN: regex::Regex = regex::Regex::new(r"(?i)^((major_brand|minor_version|compatible_brands|creation_time|handler_name|encoder)$|_|com\.)").unwrap();
    }

    BORING_PATTERN.is_match(key)
}

impl<'a> ToTags for DictionaryRef<'a> {
    fn to_tags(&self) -> Tags {
        self.iter()
            .filter_map(|(k, v)| {
                if v.is_empty() || tag_is_boring(k) {
                    None
                } else {
                    Some((k.to_string(), v.to_string()))
                }
            })
            .collect()
    }
}

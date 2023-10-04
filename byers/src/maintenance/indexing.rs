use std::path::{Path, PathBuf};

use audiotags::{AudioTagEdit, Id3v2Tag};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::maintenance::rewrite_music_path;

pub trait WavTag {
    fn read_from_wav_path(path: impl AsRef<Path>) -> Result<Self, id3::Error>
    where
        Self: Sized;
}

impl WavTag for Id3v2Tag {
    fn read_from_wav_path(path: impl AsRef<Path>) -> Result<Self, id3::Error>
    where
        Self: Sized,
    {
        let id_tag = id3::Tag::read_from_wav_path(path)?;

        Ok(id_tag.into())
    }
}

#[tracing::instrument(skip(db))]
async fn index(db: PgPool, directory: PathBuf) -> anyhow::Result<()> {
    // recursively change all file paths from directory to /music
    let files = walkdir::WalkDir::new(&directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some()
                && vec!["mp3", "flac", "ogg", "wav"].contains(
                    &e.path()
                        .extension()
                        .unwrap()
                        .to_string_lossy()
                        .to_lowercase()
                        .as_str(),
                )
        })
        .map(|e| e.path().to_owned())
        .collect::<Vec<_>>();

    // prune database
    info!("Pruning indexing database");
    sqlx::query!("DELETE FROM songs").execute(&db).await?;

    let len = files.len();
    let mut failed_files = vec![];
    for file in files {
        let result = index_file(db.clone(), &file, &directory).await;
        if let Err(e) = result {
            error!("failed to index file: {}", e);
            failed_files.push(file);
        }
    }
    info!("Indexed {} files", len);
    if !failed_files.is_empty() {
        warn!("Failed to index {} files", failed_files.len());
        warn!("Failed files: {:#?}", failed_files);
    }

    Ok(())
}

#[tracing::instrument(skip(db))]
async fn index_file(db: PgPool, path: &Path, music_path: &Path) -> anyhow::Result<()> {
    let (title, artist, album) = {
        if path.extension().unwrap().to_ascii_lowercase() == "wav" {
            let Ok(tag) = Id3v2Tag::read_from_wav_path(path) else {
                return Err(anyhow::anyhow!("failed to read wav tag"));
            };

            (
                tag.title().unwrap_or("").to_owned(),
                tag.artist().unwrap_or("").to_owned(),
                tag.album().map(|a| a.title).unwrap_or("").to_owned(),
            )
        } else {
            let tag = audiotags::Tag::new().read_from_path(path)?;
            (
                tag.title().unwrap_or("").to_owned(),
                tag.artist().unwrap_or("").to_owned(),
                tag.album().map(|a| a.title).unwrap_or("").to_owned(),
            )
        }
    };
    let meta = metadata::media_file::MediaFileMetadata::new(&path)?;
    let duration = meta._duration.unwrap_or(0_f64);
    let bitrate = meta
        ._bit_rate
        .unwrap_or((meta.file_size * 8) / duration as u64);

    let mut hasher: Sha256 = Digest::new();
    hasher.update(path.canonicalize()?.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);

    let path = rewrite_music_path(path, music_path)?;

    info!(
        "Indexing {title} by {artist} on {album} at path {}",
        path.display()
    );
    sqlx::query!(
        "INSERT INTO songs (title, artist, album, file_path, duration, file_hash, bitrate) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        title.replace(char::from(0), ""),
        artist.replace(char::from(0), ""),
        album.replace(char::from(0), ""),
        path.display().to_string(),
        duration,
        hash_str,
        bitrate as i32,
    )
    .execute(&db)
    .await?;

    for (key, value) in &meta.tags {
        sqlx::query!(
            "INSERT INTO song_tags (song_id, tag, value) VALUES ($1, $2, $3)",
            hash_str,
            key,
            value,
        )
        .execute(&db)
        .await?;
    }

    Ok(())
}

async fn drop_index(db: PgPool, path: &Path, music_path: &Path) -> anyhow::Result<()> {
    let db_path = rewrite_music_path(path, music_path)?;
    info!("Dropping index for {}", path.display());

    sqlx::query!(
        "DELETE FROM songs WHERE file_path = $1",
        db_path.display().to_string()
    )
    .execute(&db)
    .await?;

    Ok(())
}

async fn drop_index_folder(
    db: PgPool,
    folder_path: &Path,
    music_path: &Path,
) -> anyhow::Result<()> {
    let db_path = rewrite_music_path(folder_path, music_path)?;
    info!("Dropping index for {}", folder_path.display());

    sqlx::query!(
        "DELETE FROM songs WHERE file_path LIKE $1",
        format!("{}%", db_path.display().to_string())
    )
    .execute(&db)
    .await?;

    Ok(())
}

use std::path::{Path, PathBuf};

use audiotags::{AudioTagEdit, Id3v2Tag};
use sea_orm::DatabaseConnection;
use sha2::{Digest, Sha256};
use tracing::{debug, error, info, warn};

use crate::{
    controllers::song_tags::InsertParams as TagParams,
    controllers::songs::InsertParams,
    maintenance::rewrite_music_path,
    prelude::{Songs, *},
};

pub trait WavTag {
    fn read_from_wav_path(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

impl WavTag for Id3v2Tag {
    fn read_from_wav_path(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let id_tag = id3::Tag::read_from_path(path)?;

        Ok(id_tag.into())
    }
}

#[tracing::instrument(skip(db))]
pub async fn index(db: &DatabaseConnection, directory: PathBuf) -> Result<()> {
    info!("Pruning indexing database");
    Songs::prune(db).await?;

    let files = walkdir::WalkDir::new(&directory)
        .into_iter()
        .filter_map(|e| {
            if let Err(e) = &e {
                error!("Failed to walk directory: {}", e);
                None
            } else {
                e.ok()
            }
        })
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some()
                && SUPPORTED_AUDIO_FORMATS.contains(
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
    debug!("Found {} files", files.len());

    let len = files.len();
    let mut failed_files = vec![];
    for file in files {
        let result = index_file(db, &file, &directory).await;
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
pub async fn index_file(db: &DatabaseConnection, path: &Path, music_path: &Path) -> Result<()> {
    if !SUPPORTED_AUDIO_FORMATS.contains(
        &path
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
            .as_str(),
    ) {
        return Ok(());
    }

    let (title, artist, album) = {
        if path.extension().unwrap().to_ascii_lowercase() == "wav" {
            let tag = Id3v2Tag::read_from_wav_path(path)?;

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
    // let meta = metadata::media_file::MediaFileMetadata::new(&path)?;
    let meta = super::metadata::MusicMetadata::new(&path)?;

    let mut hasher: Sha256 = Digest::new();
    hasher.update(path.canonicalize()?.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);

    let path = rewrite_music_path(path, music_path)?;

    info!(
        "Indexing {title} by {artist} on {album} at path {}",
        path.display()
    );

    let song = Songs::insert(
        InsertParams {
            title: title.replace(char::from(0), ""),
            artist: artist.replace(char::from(0), ""),
            album: album.replace(char::from(0), ""),
            file_path: path.display().to_string(),
            file_hash: hash_str.clone(),
            duration: meta.duration,
            bitrate: meta.bitrate as i32,
        },
        db,
    )
    .await?;

    Tags::insert_many(
        &song,
        &meta
            .tags
            .into_iter()
            .map(|(k, v)| TagParams { tag: k, value: v })
            .collect::<Vec<_>>(),
        db,
    )
    .await?;

    Ok(())
}

pub async fn drop_index(db: &DatabaseConnection, path: &Path, music_path: &Path) -> Result<()> {
    let db_path = rewrite_music_path(path, music_path)?;
    info!("Dropping index for {}", path.display());

    Songs::delete_by_path(&db_path, db).await?;

    Ok(())
}

pub async fn drop_index_folder(
    db: &DatabaseConnection,
    folder_path: &Path,
    music_path: &Path,
) -> Result<()> {
    let db_path = rewrite_music_path(folder_path, music_path)?;
    info!("Dropping index for {}", folder_path.display());

    let songs = Songs::get_by_directory(&db_path, db).await?;
    for song in songs {
        song.delete(db).await?;
    }

    Ok(())
}

pub async fn create_playlist(db: &DatabaseConnection, playlist_path: &Path) -> Result<()> {
    let songs = Songs::get_all_paths(db)
        .await?
        .into_iter()
        .map(m3u::path_entry)
        .collect::<Vec<_>>();

    let mut file = std::fs::File::create(playlist_path)?;
    let mut writer = m3u::Writer::new(&mut file);
    for entry in songs {
        writer.write_entry(&entry)?;
    }

    Ok(())
}

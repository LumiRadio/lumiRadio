use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use audiotags::{AudioTagEdit, Id3v2Tag};
use clap::{Parser, Subcommand};
use notify::Watcher;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::sync::{mpsc::Receiver, Mutex};
use tracing::{debug, error, info, warn};

use crate::custom_metadata::WavTag;

mod custom_metadata;

#[derive(Parser)]
#[command(author, about, version)]
struct CliArgs {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    HouseKeeping(HouseKeeping),
    Indexing(Indexing),
    Import(Import),
}

#[derive(Parser, Debug, Clone)]
struct HouseKeeping {
    #[clap(short, long)]
    dry_run: bool,
    #[clap(short = 'D', long)]
    database_url: String,

    music_path: PathBuf,
}

#[derive(Parser, Debug, Clone)]
struct Indexing {
    #[clap(short, long)]
    dry_run: bool,
    #[clap(short = 'D', long)]
    database_url: String,

    path: PathBuf,
}

#[derive(Parser, Debug, Clone)]
struct Import {
    #[clap(short, long)]
    dry_run: bool,
    #[clap(short = 'D', long)]
    database_url: String,

    #[clap(subcommand)]
    subcmd: ImportSubCommand,
}

#[derive(Subcommand, Debug, Clone)]
enum ImportSubCommand {
    Streamlabs(StreamlabsImport),
}

#[derive(Parser, Debug, Clone)]
struct StreamlabsImport {
    path: PathBuf,
}

fn rewrite_music_path(path: &Path, music_path: &Path) -> anyhow::Result<PathBuf> {
    Ok(Path::new("/music").join(path.strip_prefix(music_path)?))
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
        "INSERT INTO songs (title, artist, album, file_path, duration, file_hash) VALUES ($1, $2, $3, $4, $5, $6)",
        title.replace(char::from(0), ""),
        artist.replace(char::from(0), ""),
        album.replace(char::from(0), ""),
        path.display().to_string(),
        duration,
        hash_str,
    )
    .execute(&db)
    .await?;

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

fn async_watcher(
    handle: tokio::runtime::Handle,
) -> anyhow::Result<(
    notify::RecommendedWatcher,
    Receiver<notify::Result<notify::event::Event>>,
)> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let tx = Arc::new(Mutex::new(tx));

    let watcher = notify::RecommendedWatcher::new(
        move |res| {
            debug!("received event: {:?}", res);
            let tx_clone = Arc::clone(&tx);
            handle.spawn(async move {
                debug!("sending event");
                let tx = tx_clone.lock().await;
                tx.send(res).await.unwrap();
            });
        },
        notify::Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P, watcher_pool: PgPool) -> anyhow::Result<()> {
    let tokio_rt = tokio::runtime::Handle::current();
    let (mut watcher, mut rx) = async_watcher(tokio_rt)?;
    watcher.watch(path.as_ref(), notify::RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        let event: notify::event::Event = match res {
            Ok(event) => event,
            Err(e) => {
                error!("watch error: {}", e);
                continue;
            }
        };

        match &event.kind {
            notify::event::EventKind::Access(notify::event::AccessKind::Close(
                notify::event::AccessMode::Write,
            )) => {
                debug!("file written: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();
                let watcher_pool = watcher_pool.clone();
                index_file(watcher_pool, file_path, path.as_ref())
                    .await
                    .unwrap();
            }
            notify::event::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::From,
            )) => {
                debug!("file modified: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();
                drop_index(watcher_pool.clone(), file_path, path.as_ref())
                    .await
                    .unwrap();
            }
            notify::event::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::To,
            )) => {
                debug!("file modified: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();
                index_file(watcher_pool.clone(), file_path, path.as_ref())
                    .await
                    .unwrap();
            }
            notify::event::EventKind::Remove(notify::event::RemoveKind::File) => {
                debug!("file removed: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();
                drop_index(watcher_pool.clone(), file_path, path.as_ref())
                    .await
                    .unwrap();
            }
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = CliArgs::parse();

    match args.subcmd {
        SubCommand::Indexing(indexing) => {
            debug!("indexing");
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect_lazy(&indexing.database_url)?;

            index(pool, indexing.path).await?;
        }
        SubCommand::HouseKeeping(house_keeping) => {
            debug!("house keeping");
            // this is a continous list of tasks which runs forever
            // it should check the filesystem for new files
            // if they are new, index them into the database
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect_lazy(&house_keeping.database_url)?;

            let tasks = vec![async_watch(house_keeping.music_path.clone(), pool.clone())];

            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            for task in tasks {
                let tx = tx.clone();
                debug!("spawning task");
                tokio::spawn(async move {
                    let result = task.await;
                    if let Err(e) = result {
                        error!("task failed: {}", e);
                    }
                    tx.send(()).await.unwrap();
                });
            }

            // anything else that doesn't need tasks

            while (rx.recv().await).is_some() {
                debug!("received");
            }
        }
        SubCommand::Import(_) => {}
    }

    Ok(())
}

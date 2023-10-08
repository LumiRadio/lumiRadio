use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::{Parser, Subcommand};
use judeharley::{db::DbSong, PgPool};
use notify::Watcher;
use tokio::sync::{mpsc::Receiver, Mutex};
use tracing::{debug, error};

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
                judeharley::maintenance::indexing::index_file(
                    watcher_pool,
                    file_path,
                    path.as_ref(),
                )
                .await?;
            }
            notify::event::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::From,
            )) => {
                debug!("file modified: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();

                if file_path.is_file() {
                    judeharley::maintenance::indexing::drop_index(
                        watcher_pool.clone(),
                        file_path,
                        path.as_ref(),
                    )
                    .await
                    .unwrap();
                } else if file_path.is_dir() {
                    judeharley::maintenance::indexing::drop_index_folder(
                        watcher_pool.clone(),
                        file_path,
                        path.as_ref(),
                    )
                    .await
                    .unwrap();
                }
            }
            notify::event::EventKind::Modify(notify::event::ModifyKind::Name(
                notify::event::RenameMode::To,
            )) => {
                debug!("file modified: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();

                if file_path.is_file() {
                    judeharley::maintenance::indexing::index_file(
                        watcher_pool.clone(),
                        file_path,
                        path.as_ref(),
                    )
                    .await
                    .unwrap();
                } else if file_path.is_dir() {
                    for entry in walkdir::WalkDir::new(file_path) {
                        let entry = entry.unwrap();
                        if entry.file_type().is_file() {
                            judeharley::maintenance::indexing::index_file(
                                watcher_pool.clone(),
                                entry.path(),
                                path.as_ref(),
                            )
                            .await
                            .unwrap();
                        }
                    }
                }
            }
            notify::event::EventKind::Remove(notify::event::RemoveKind::File) => {
                debug!("file removed: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();
                judeharley::maintenance::indexing::drop_index(
                    watcher_pool.clone(),
                    file_path,
                    path.as_ref(),
                )
                .await
                .unwrap();
            }
            notify::event::EventKind::Remove(notify::event::RemoveKind::Folder) => {
                debug!("folder removed: {:?}", event.paths);
                let file_path = event.paths.first().unwrap();
                judeharley::maintenance::indexing::drop_index_folder(
                    watcher_pool.clone(),
                    file_path,
                    path.as_ref(),
                )
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
            let pool = judeharley::connect_database(&indexing.database_url).await?;

            judeharley::maintenance::indexing::index(pool, indexing.path).await?;
        }
        SubCommand::HouseKeeping(house_keeping) => {
            debug!("house keeping");
            // this is a continous list of tasks which runs forever
            // it should check the filesystem for new files
            // if they are new, index them into the database
            let pool = judeharley::connect_database(&house_keeping.database_url).await?;

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

use std::path::{PathBuf, Path};

use audiotags::Tag;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, about, version)]
struct CliArgs {
    /// The subcommand to run
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Index the database with songs
    Index(IndexArgs),
}

#[derive(Parser, Debug)]
struct IndexArgs {
    /// The database URL to connect to
    #[clap(short, long)]
    database_url: String,

    /// The directory to index
    directory: PathBuf,
}

async fn index(database_url: String, directory: PathBuf) {
    println!("Indexing {} into {}", directory.display(), database_url);

    let db = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");
    // recursively change all file paths from directory to /music
    let files = walkdir::WalkDir::new(&directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_owned())
        .collect::<Vec<_>>();

    // prune database
    sqlx::query!("DELETE FROM songs")
        .execute(&db)
        .await
        .expect("failed to delete songs from database");
    
    for file in files {
        let tag = Tag::new().read_from_path(&file).unwrap();
        let title = tag.title().unwrap_or("");
        let artist = tag.artist().unwrap_or("");
        let album = tag.album().unwrap_or(audiotags::Album { title: "", artist: None, cover: None });
        let meta = metadata::media_file::MediaFileMetadata::new(&file).unwrap();
        let duration = meta._duration.unwrap_or(0_f64);

        let new_path = Path::new("/music").join(file.strip_prefix(&directory).unwrap());

        println!("Indexing {title} by {artist} on {} at path {}", album.title, file.display());
        sqlx::query!("INSERT INTO songs (title, artist, album, file_path, duration) VALUES ($1, $2, $3, $4, $5)", title, artist, album.title, new_path.display().to_string(), duration)
            .execute(&db)
            .await
            .expect("failed to insert song into database");
    }
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let CliArgs { subcmd } = args;

    match subcmd {
        SubCommand::Index(args) => {
            let IndexArgs { database_url, directory } = args;
            index(database_url, directory).await;
        }
    }
}

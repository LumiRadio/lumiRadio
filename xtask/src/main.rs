use std::path::{Path, PathBuf};

use audiotags::Tag;
use clap::{Parser, Subcommand};

mod slcb;

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

    /// Import old bot data
    Import(ImportArgs),
}

#[derive(Parser, Debug)]
struct IndexArgs {
    /// The database URL to connect to
    #[clap(short, long)]
    database_url: String,

    /// The directory to index
    directory: PathBuf,
}

#[derive(Parser, Debug)]
struct ImportArgs {
    /// The database URL to connect to
    #[clap(short, long)]
    database_url: String,

    /// The directory to import
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
        let album = tag.album().unwrap_or(audiotags::Album {
            title: "",
            artist: None,
            cover: None,
        });
        let meta = metadata::media_file::MediaFileMetadata::new(&file).unwrap();
        let duration = meta._duration.unwrap_or(0_f64);

        let new_path = Path::new("/music").join(file.strip_prefix(&directory).unwrap());

        println!(
            "Indexing {title} by {artist} on {} at path {}",
            album.title,
            file.display()
        );
        sqlx::query!("INSERT INTO songs (title, artist, album, file_path, duration) VALUES ($1, $2, $3, $4, $5)", title, artist, album.title, new_path.display().to_string(), duration)
            .execute(&db)
            .await
            .expect("failed to insert song into database");
    }
}

async fn import(database_url: String, directory: PathBuf) {
    let db = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");
    let user_data_json = directory.join("data_export.json");
    let ranks_csv = directory.join("Ranks.csv");
    let byers_plus_db = directory.join("byers_plus.db");
    let byers_plus_db = sqlite::open(&byers_plus_db).expect("failed to open byers_plus.db");

    let currency_entries = slcb::UserDataEntry::load(&user_data_json);
    let ranks_entries = slcb::RanksCsvEntry::load(&ranks_csv);
    let placeholder_entries = slcb::PlaceholderEntry::load(&byers_plus_db);

    println!("Truncating tables");
    sqlx::query!("DELETE FROM slcb_currency")
        .execute(&db)
        .await
        .expect("failed to delete currency from database");
    sqlx::query!("DELETE FROM slcb_rank")
        .execute(&db)
        .await
        .expect("failed to delete ranks from database");
    sqlx::query!("DELETE FROM bp_counters")
        .execute(&db)
        .await
        .expect("failed to delete counters from database");

    println!("Importing currency data");
    for entry in currency_entries {
        sqlx::query!(
            "INSERT INTO slcb_currency (username, points, hours, user_id) VALUES ($1, $2, $3, $4)",
            entry.user_name,
            entry.points,
            entry.time_watched / 60,
            entry.user_id
        )
        .execute(&db)
        .await
        .expect("failed to insert currency into database");
    }

    println!("Importing rank data");
    for entry in ranks_entries {
        sqlx::query!(
            "INSERT INTO slcb_rank (rank_name, hour_requirement, channel_id) VALUES ($1, $2, $3)",
            entry.name,
            entry.hour_requirement,
            if entry.channel_id.is_empty() {
                None
            } else {
                Some(entry.channel_id)
            }
        )
        .execute(&db)
        .await
        .expect("failed to insert rank into database");
    }

    println!("Importing placeholder data");
    for entry in placeholder_entries {
        sqlx::query!(
            "INSERT INTO bp_counters (constant, value) VALUES ($1, $2)",
            entry.key,
            entry.value
        )
        .execute(&db)
        .await
        .expect("failed to insert counter into database");
    }

    println!("Done!");
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let CliArgs { subcmd } = args;

    match subcmd {
        SubCommand::Index(args) => {
            let IndexArgs {
                database_url,
                directory,
            } = args;
            index(database_url, directory).await;
        }
        SubCommand::Import(args) => {
            let ImportArgs {
                database_url,
                directory,
            } = args;
            import(database_url, directory).await;
        }
    }
}

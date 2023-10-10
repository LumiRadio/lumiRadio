use std::path::{Path, PathBuf};

use audiotags::Tag;
use clap::{Parser, Subcommand};
use serde_json::Value;

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
    /// Register Discord linked role metadata
    Register(RegisterArgs),
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

#[derive(Parser, Debug)]
struct RegisterArgs {
    /// The Discord bot token
    #[clap(short, long)]
    token: String,

    /// The Discord client ID
    #[clap(short, long)]
    client_id: String,

    /// The file with linked roles (in JSON format)
    file: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let CliArgs { subcmd } = args;

    match subcmd {
        SubCommand::Register(args) => {
            let file_json = std::fs::read_to_string(args.file).expect("failed to read file");
            let linked_roles: Value =
                serde_json::from_str(&file_json).expect("failed to parse JSON");
            let client = reqwest::Client::new();
            let endpoint = format!(
                "https://discord.com/api/v10/applications/{}/role-connections/metadata",
                &args.client_id
            );

            println!("Registering linked roles at {}", endpoint);
            client
                .put(&endpoint)
                .bearer_auth(&args.token)
                .json(&linked_roles)
                .send()
                .await
                .expect("failed to send request");
            println!("Done!");
        }
    }
}

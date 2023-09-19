use std::path::Path;

use serde::Deserialize;
use sqlite::Connection;

#[derive(Deserialize, Debug, Clone)]
pub struct UserDataEntry {
    pub user_name: String,
    pub points: i32,
    pub time_watched: i32,
    pub rank: String,
    pub user_id: String,
}

impl UserDataEntry {
    pub fn load(path: &Path) -> Vec<Self> {
        let mut file = std::fs::File::open(path).expect("failed to open user data json");
        serde_json::from_reader(&mut file).expect("failed to deserialize user data json")
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CurrencyCsvEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Points")]
    pub points: i32,
    #[serde(rename = "Hours")]
    pub hours: i32,
}

impl CurrencyCsvEntry {
    pub fn load(path: &Path) -> Vec<Self> {
        let mut rdr = csv::Reader::from_path(path).expect("failed to open currency csv");
        let mut entries = Vec::new();

        for result in rdr.deserialize() {
            let entry: Self = result.expect("failed to deserialize currency csv");
            entries.push(entry);
        }
        entries
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RanksCsvEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Requirement")]
    pub hour_requirement: i32,
    #[serde(rename = "UserGroup")]
    pub user_group: String,
    #[serde(rename = "Info")]
    pub channel_id: String,
}

impl RanksCsvEntry {
    pub fn load(path: &Path) -> Vec<Self> {
        let mut rdr = csv::Reader::from_path(path).expect("failed to open ranks csv");
        let mut entries = Vec::new();

        for result in rdr.deserialize() {
            let entry: Self = result.expect("failed to deserialize ranks csv");
            entries.push(entry);
        }
        entries
    }
}

pub struct PlaceholderEntry {
    pub key: String,
    pub value: i32,
}

impl PlaceholderEntry {
    pub fn load(connection: &Connection) -> Vec<Self> {
        let mut stmt = connection
            .prepare("SELECT * FROM bp_placeholders")
            .expect("failed to prepare statement");
        let mut entries = Vec::new();

        while let sqlite::State::Row = stmt.next().unwrap() {
            let key = stmt.read::<String, _>(0).unwrap();
            let value = stmt.read::<i64, _>(1).unwrap();
            entries.push(Self {
                key,
                value: value as i32,
            });
        }
        entries
    }
}

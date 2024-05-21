use chrono::NaiveDateTime;
use sea_orm::{prelude::*, FromQueryResult, IntoActiveModel, QueryOrder, QuerySelect, Set};

use crate::controllers::CountQuery;
use crate::custom_entities::songs::Model as SongModel;
use crate::discord::DiscordConnection;
use crate::entities::cans::{Column as CanColumn, Entity as Can};
use crate::entities::connected_youtube_accounts::{
    Entity as ConnectedYoutubeAccount, Model as ConnectedYoutubeAccountModel,
};
use crate::entities::favourite_songs::Model as FavouriteSongModel;
use crate::{entities::users::*, JudeHarleyError};

#[derive(Debug, Clone, Default)]
pub struct UpdateParams {
    pub watched_time: Option<sea_orm::entity::prelude::Decimal>,
    pub last_message_sent: Option<NaiveDateTime>,
    pub boonbucks: Option<u32>,
    pub migrated: Option<bool>,
    pub amber: Option<u32>,
    pub amethyst: Option<u32>,
    pub artifact: Option<u32>,
    pub caulk: Option<u32>,
    pub chalk: Option<u32>,
    pub cobalt: Option<u32>,
    pub diamond: Option<u32>,
    pub garnet: Option<u32>,
    pub gold: Option<u32>,
    pub iodine: Option<u32>,
    pub marble: Option<u32>,
    pub mercury: Option<u32>,
    pub quartz: Option<u32>,
    pub ruby: Option<u32>,
    pub rust: Option<u32>,
    pub shale: Option<u32>,
    pub sulfur: Option<u32>,
    pub tar: Option<u32>,
    pub uranium: Option<u32>,
    pub zillium: Option<u32>,
}

#[derive(FromQueryResult)]
struct UserCount {
    count: i64,
}

impl Model {
    pub async fn get(id: u64, db: &DatabaseConnection) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(id as i64)
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_or_insert(id: u64, db: &DatabaseConnection) -> Result<Self, JudeHarleyError> {
        if let Some(user) = Self::get(id, db).await? {
            return Ok(user);
        }

        let user = ActiveModel {
            id: Set(id as i64),
            ..Default::default()
        };

        user.insert(db).await.map_err(Into::into)
    }

    pub async fn get_with_at_least_n_hours(
        min: i32,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::WatchedTime.gte::<sea_orm::entity::prelude::Decimal>(min.into()))
            .order_by(Column::WatchedTime, sea_orm::Order::Desc)
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn hour_position(&self, db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        Entity::find()
            .select_only()
            .column_as(Column::Id.count(), "count")
            .filter(Column::WatchedTime.gt(self.watched_time))
            .into_model::<UserCount>()
            .one(db)
            .await
            .map_err(Into::into)
            .map(|count| count.map(|c| c.count + 1).unwrap_or(1))
    }

    pub async fn boondollar_position(
        &self,
        db: &DatabaseConnection,
    ) -> Result<i64, JudeHarleyError> {
        Entity::find()
            .select_only()
            .column_as(Column::Boonbucks.count(), "count")
            .filter(Column::Boonbucks.gt(self.boonbucks))
            .into_model::<UserCount>()
            .one(db)
            .await
            .map_err(Into::into)
            .map(|count| count.map(|c| c.count + 1).unwrap_or(1))
    }

    pub async fn cans(&self, db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        self.find_related(Can)
            .select_only()
            .column_as(CanColumn::Id.count(), "count")
            .filter(CanColumn::Legit.eq(true))
            .into_model::<CountQuery>()
            .one(db)
            .await
            .map_err(Into::into)
            .map(|count| count.map(|c| c.count).unwrap_or(0))
    }

    pub async fn place_can(&self, db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        crate::entities::cans::Model::insert(self, true, db)
            .await
            .map_err(Into::into)
    }

    pub async fn insert_channels(
        &self,
        channels: &[DiscordConnection],
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        ConnectedYoutubeAccountModel::insert_many(self, channels, db).await
    }

    pub async fn linked_channels(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<ConnectedYoutubeAccountModel>, JudeHarleyError> {
        self.find_related(ConnectedYoutubeAccount)
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        self,
        params: UpdateParams,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        let mut user = self.into_active_model();

        if let Some(watched_time) = params.watched_time {
            user.watched_time = Set(watched_time);
        }
        if let Some(boonbucks) = params.boonbucks {
            user.boonbucks = Set(boonbucks as i32);
        }
        if let Some(migrated) = params.migrated {
            user.migrated = Set(migrated);
        }
        if let Some(amber) = params.amber {
            user.amber = Set(amber as i32);
        }
        if let Some(amethyst) = params.amethyst {
            user.amethyst = Set(amethyst as i32);
        }
        if let Some(artifact) = params.artifact {
            user.artifact = Set(artifact as i32);
        }
        if let Some(caulk) = params.caulk {
            user.caulk = Set(caulk as i32);
        }
        if let Some(chalk) = params.chalk {
            user.chalk = Set(chalk as i32);
        }
        if let Some(cobalt) = params.cobalt {
            user.cobalt = Set(cobalt as i32);
        }
        if let Some(diamond) = params.diamond {
            user.diamond = Set(diamond as i32);
        }
        if let Some(garnet) = params.garnet {
            user.garnet = Set(garnet as i32);
        }
        if let Some(gold) = params.gold {
            user.gold = Set(gold as i32);
        }
        if let Some(iodine) = params.iodine {
            user.iodine = Set(iodine as i32);
        }
        if let Some(marble) = params.marble {
            user.marble = Set(marble as i32);
        }
        if let Some(mercury) = params.mercury {
            user.mercury = Set(mercury as i32);
        }
        if let Some(quartz) = params.quartz {
            user.quartz = Set(quartz as i32);
        }
        if let Some(ruby) = params.ruby {
            user.ruby = Set(ruby as i32);
        }
        if let Some(rust) = params.rust {
            user.rust = Set(rust as i32);
        }
        if let Some(shale) = params.shale {
            user.shale = Set(shale as i32);
        }
        if let Some(sulfur) = params.sulfur {
            user.sulfur = Set(sulfur as i32);
        }
        if let Some(tar) = params.tar {
            user.tar = Set(tar as i32);
        }
        if let Some(uranium) = params.uranium {
            user.uranium = Set(uranium as i32);
        }
        if let Some(zillium) = params.zillium {
            user.zillium = Set(zillium as i32);
        }
        if let Some(last_message_sent) = params.last_message_sent {
            user.last_message_sent = Set(Some(last_message_sent));
        }

        user.updated_at = Set(chrono::Utc::now().naive_utc());
        user.update(db).await.map_err(Into::into)
    }

    pub async fn favourite_song(
        &self,
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        if FavouriteSongModel::get_by_user_and_song(self, song, db)
            .await?
            .is_some()
        {
            return Ok(());
        }

        FavouriteSongModel::insert(song, self, db).await
    }

    pub async fn unfavourite_song(
        &self,
        song: &SongModel,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        FavouriteSongModel::delete_by_user_and_song(self, song, db).await
    }

    pub async fn list_favourites(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<SongModel>, JudeHarleyError> {
        FavouriteSongModel::get_by_user(self, db).await
    }
}

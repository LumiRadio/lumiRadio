use sea_orm::{prelude::*, FromQueryResult, QueryOrder, QuerySelect, Set};

use crate::controllers::CountQuery;
use crate::custom_entities::songs::Model as SongModel;
use crate::discord::DiscordConnection;
use crate::entities::cans::{Column as CanColumn, Entity as Can};
use crate::entities::connected_youtube_accounts::{
    Entity as ConnectedYoutubeAccount, Model as ConnectedYoutubeAccountModel,
};
use crate::entities::favourite_songs::Model as FavouriteSongModel;
use crate::{entities::users::*, JudeHarleyError};

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
        &self,
        params: ActiveModel,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        Entity::update(params)
            .filter(Column::Id.eq(self.id))
            .exec(db)
            .await
            .map_err(Into::into)
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

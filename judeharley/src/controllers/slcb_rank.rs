use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, QuerySelect, prelude::*};

use crate::entities::{slcb_rank::*, users::Model as UserModel};
use crate::prelude::JudeHarleyError;

impl Model {
    pub async fn get_by_id(
        id: i32,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(id).one(db).await.map_err(Into::into)
    }

    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find().all(db).await.map_err(Into::into)
    }

    pub async fn create(
        rank_name: &str,
        hour_requirement: i32,
        channel_id: Option<&str>,
        user_id: Option<i64>,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        let rank = ActiveModel {
            rank_name: ActiveValue::set(rank_name.to_string()),
            hour_requirement: ActiveValue::set(hour_requirement),
            channel_id: ActiveValue::set(channel_id.map(|s| s.to_string())),
            user_id: ActiveValue::set(user_id),
            ..Default::default()
        };
        rank.insert(db).await.map_err(Into::into)
    }

    pub async fn delete(id: i32, db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        Entity::delete_by_id(id)
            .exec(db)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn update(
        id: i32,
        rank_name: Option<&str>,
        hour_requirement: Option<i32>,
        channel_id: Option<&str>,
        user_id: Option<i64>,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        let mut rank = Self::get_by_id(id, db)
            .await?
            .ok_or(JudeHarleyError::RankNotFound)?
            .into_active_model();
        if let Some(rank_name) = rank_name {
            rank.rank_name = ActiveValue::set(rank_name.to_string());
        }
        if let Some(hour_requirement) = hour_requirement {
            rank.hour_requirement = ActiveValue::set(hour_requirement);
        }
        if let Some(channel_id) = channel_id {
            rank.channel_id = ActiveValue::set(Some(channel_id.to_string()));
        }
        if let Some(user_id) = user_id {
            rank.user_id = ActiveValue::set(Some(user_id));
        }
        rank.update(db).await.map_err(Into::into)
    }

    pub async fn get_rank_for_user(
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<String, JudeHarleyError> {
        let linked_channels = user
            .linked_channels(db)
            .await?
            .into_iter()
            .map(|c| c.youtube_channel_id)
            .collect::<Vec<_>>();

        let rank = Entity::find()
            .filter(Column::HourRequirement.lte(user.watched_time / 3600))
            .filter(
                Column::ChannelId
                    .is_null()
                    .or(Column::ChannelId.is_in(linked_channels))
                    .or(Column::UserId.eq(user.id)),
            )
            .order_by_desc(Column::HourRequirement)
            .limit(1)
            .one(db)
            .await?;

        Ok(rank
            .map(|r| r.rank_name)
            .unwrap_or("Wow, literally no rank available...".to_string()))
    }

    pub async fn get_next_rank_for_user(
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        let linked_channels = user
            .linked_channels(db)
            .await?
            .into_iter()
            .map(|c| c.youtube_channel_id)
            .collect::<Vec<_>>();

        Entity::find()
            .filter(Column::HourRequirement.gt(user.watched_time / 3600))
            .filter(
                Column::ChannelId
                    .is_null()
                    .or(Column::ChannelId.is_in(linked_channels))
                    .or(Column::UserId.eq(user.id)),
            )
            .order_by_asc(Column::HourRequirement)
            .limit(1)
            .one(db)
            .await
            .map_err(Into::into)
    }
}

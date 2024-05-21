use sea_orm::{prelude::*, QueryOrder, QuerySelect};

use crate::entities::{slcb_rank::*, users::Model as UserModel};
use crate::prelude::JudeHarleyError;

impl Model {
    pub async fn get_rank_for_user(
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<String, JudeHarleyError> {
        let watched_time: i32 = user
            .watched_time
            .clone()
            .trunc_with_scale(0)
            .try_into()
            .expect("Invalid watched time");
        let linked_channels = user
            .linked_channels(db)
            .await?
            .into_iter()
            .map(|c| c.youtube_channel_id)
            .collect::<Vec<_>>();

        let rank = Entity::find()
            .filter(Column::HourRequirement.lte(watched_time))
            .filter(
                Column::ChannelId
                    .is_null()
                    .or(Column::ChannelId.is_in(linked_channels)),
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
        let watched_time: i32 = user
            .watched_time
            .clone()
            .trunc_with_scale(0)
            .try_into()
            .expect("Invalid watched time");
        let linked_channels = user
            .linked_channels(db)
            .await?
            .into_iter()
            .map(|c| c.youtube_channel_id)
            .collect::<Vec<_>>();

        Entity::find()
            .filter(Column::HourRequirement.gt(watched_time))
            .filter(
                Column::ChannelId
                    .is_null()
                    .or(Column::ChannelId.is_in(linked_channels)),
            )
            .order_by_asc(Column::HourRequirement)
            .limit(1)
            .one(db)
            .await
            .map_err(Into::into)
    }
}

use sea_orm::{prelude::*, Set};

use crate::discord::DiscordConnection;
use crate::entities::connected_youtube_accounts::*;
use crate::entities::users::Model as UserModel;
use crate::JudeHarleyError;

impl Model {
    pub async fn get_all(
        user: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::UserId.eq(user.id))
            .all(db)
            .await
            .map_err(Into::into)
    }

    pub async fn insert_many(
        user: &UserModel,
        channels: &[DiscordConnection],
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user.id))
            .exec(db)
            .await?;

        Entity::insert_many(
            channels
                .iter()
                .map(|c| ActiveModel {
                    user_id: Set(user.id),
                    youtube_channel_id: Set(c.id.clone()),
                    youtube_channel_name: Set(c.name.clone()),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        )
        .exec(db)
        .await?;

        Ok(())
    }
}

use sea_orm::{ActiveValue, Set, prelude::*};

use crate::{JudeHarleyError, entities::token_storage::*};

pub struct NewStoredToken {
    pub user_id: i64,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: chrono::NaiveDateTime,
}

impl Model {
    pub async fn insert(
        params: NewStoredToken,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        ActiveModel {
            user_id: Set(params.user_id),
            access_token: Set(params.access_token),
            refresh_token: Set(params.refresh_token),
            expires_at: Set(params.expires_at),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn get_by_user_id(
        user_id: u64,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        &self,
        mut params: ActiveModel,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        params.id = ActiveValue::unchanged(self.id);

        Entity::update(params)
            .filter(Column::Id.eq(self.id))
            .exec(db)
            .await
            .map_err(Into::into)
    }

    pub async fn upsert(
        params: NewStoredToken,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        if let Some(stored_token) = Self::get_by_user_id(params.user_id as u64, db).await? {
            stored_token
                .update(
                    ActiveModel {
                        id: ActiveValue::unchanged(stored_token.id),
                        user_id: ActiveValue::unchanged(stored_token.user_id),
                        access_token: Set(params.access_token),
                        refresh_token: Set(params.refresh_token),
                        expires_at: Set(params.expires_at),
                    },
                    db,
                )
                .await
        } else {
            Self::insert(params, db).await
        }
    }
}

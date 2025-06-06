use sea_orm::{Set, prelude::*};

use crate::{JudeHarleyError, entities::api_keys::*};

pub struct NewApiKey {
    pub user_id: i64,
    pub key: String,
    pub description: String,
}

impl Model {
    pub async fn insert(
        params: NewApiKey,
        db: &DatabaseConnection,
    ) -> Result<Self, JudeHarleyError> {
        ActiveModel {
            user_id: Set(params.user_id),
            key: Set(params.key.clone()),
            description: Set(params.description.clone()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(Into::into)
    }

    pub async fn revoke(&self, db: &DatabaseConnection) -> Result<(), JudeHarleyError> {
        Entity::delete_by_id(self.id).exec(db).await?;

        Ok(())
    }

    pub async fn get_by_key(
        key: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::Key.eq(key))
            .one(db)
            .await
            .map_err(Into::into)
    }
}

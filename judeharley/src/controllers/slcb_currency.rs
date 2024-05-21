use sea_orm::{prelude::*, sea_query::extension::postgres::PgExpr};

use crate::entities::slcb_currency::*;
use crate::prelude::JudeHarleyError;

impl Model {
    pub async fn get(id: i32, db: &DatabaseConnection) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find_by_id(id).one(db).await.map_err(Into::into)
    }

    pub async fn get_by_user_id(
        user_id: &str,
        db: &DatabaseConnection,
    ) -> Result<Option<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(Into::into)
    }

    pub async fn search(
        username: &str,
        db: &DatabaseConnection,
    ) -> Result<Vec<Self>, JudeHarleyError> {
        Entity::find()
            .filter(Expr::col(Column::Username).ilike(format!("%{username}%")))
            .all(db)
            .await
            .map_err(Into::into)
    }
}

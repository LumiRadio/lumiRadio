use sea_orm::sea_query::Query;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect, Set,
};

use crate::entities::cans::*;
use crate::entities::users::Model as UserModel;
use crate::JudeHarleyError;

#[derive(FromQueryResult)]
struct CanCount {
    count: i64,
}

impl Model {
    pub async fn insert(
        added_by: &UserModel,
        legit: bool,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        ActiveModel {
            added_by: Set(added_by.id),
            legit: Set(legit),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(())
    }

    pub async fn insert_n(
        added_by: &UserModel,
        amount: i64,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        let mut cans = Vec::new();

        for _ in 0..amount {
            cans.push(ActiveModel {
                added_by: Set(added_by.id),
                legit: Set(false),
                ..Default::default()
            });
        }

        Entity::insert_many(cans)
            .on_empty_do_nothing()
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn count(db: &DatabaseConnection) -> Result<i64, JudeHarleyError> {
        let count = Entity::find()
            .select_only()
            .column_as(Column::Id.count(), "count")
            .into_model::<CanCount>()
            .one(db)
            .await?;

        Ok(count.map(|c| c.count).unwrap_or(0))
    }

    pub async fn remove_last_n(
        amount: i64,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        let current = Self::count(db).await?;

        if amount > current && amount > 0 {
            return Ok(());
        }

        Entity::delete_many()
            .filter(
                Column::Id.in_subquery(Query::select().column(Column::Id).from(Entity).to_owned()),
            )
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn set(
        added_by: &UserModel,
        amount: i64,
        db: &DatabaseConnection,
    ) -> Result<(), JudeHarleyError> {
        let current = Self::count(db).await?;

        if amount < 0 {
            return Ok(());
        }

        if amount <= current {
            Self::remove_last_n(current - amount, db)
                .await
                .map_err(Into::into)
        } else {
            Self::insert_n(added_by, amount - current, db)
                .await
                .map_err(Into::into)
        }
    }
}

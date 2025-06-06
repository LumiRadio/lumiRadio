use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

fn grist_col<T: IntoIden>(name: T) -> ColumnDef {
    ColumnDef::new(name)
        .integer()
        .not_null()
        .default(0)
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Amber)
                    .drop_column(Users::Amethyst)
                    .drop_column(Users::Artifact)
                    .drop_column(Users::Caulk)
                    .drop_column(Users::Chalk)
                    .drop_column(Users::Cobalt)
                    .drop_column(Users::Diamond)
                    .drop_column(Users::Garnet)
                    .drop_column(Users::Gold)
                    .drop_column(Users::Iodine)
                    .drop_column(Users::Marble)
                    .drop_column(Users::Mercury)
                    .drop_column(Users::Quartz)
                    .drop_column(Users::Ruby)
                    .drop_column(Users::Rust)
                    .drop_column(Users::Shale)
                    .drop_column(Users::Sulfur)
                    .drop_column(Users::Tar)
                    .drop_column(Users::Uranium)
                    .drop_column(Users::Zillium)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(grist_col(Users::Amber))
                    .add_column(grist_col(Users::Amethyst))
                    .add_column(grist_col(Users::Artifact))
                    .add_column(grist_col(Users::Caulk))
                    .add_column(grist_col(Users::Chalk))
                    .add_column(grist_col(Users::Cobalt))
                    .add_column(grist_col(Users::Diamond))
                    .add_column(grist_col(Users::Garnet))
                    .add_column(grist_col(Users::Gold))
                    .add_column(grist_col(Users::Iodine))
                    .add_column(grist_col(Users::Marble))
                    .add_column(grist_col(Users::Mercury))
                    .add_column(grist_col(Users::Quartz))
                    .add_column(grist_col(Users::Ruby))
                    .add_column(grist_col(Users::Rust))
                    .add_column(grist_col(Users::Shale))
                    .add_column(grist_col(Users::Sulfur))
                    .add_column(grist_col(Users::Tar))
                    .add_column(grist_col(Users::Uranium))
                    .add_column(grist_col(Users::Zillium))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Amber,
    Amethyst,
    Artifact,
    Caulk,
    Chalk,
    Cobalt,
    Diamond,
    Garnet,
    Gold,
    Iodine,
    Marble,
    Mercury,
    Quartz,
    Ruby,
    Rust,
    Shale,
    Sulfur,
    Tar,
    Uranium,
    Zillium,
}

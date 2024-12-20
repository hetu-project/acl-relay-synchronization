use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LastUpdate::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LastUpdate::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LastUpdate::LastUpdate)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LastUpdate::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LastUpdate::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LastUpdate {
    Table,
    Id,
    LastUpdate,
    UpdatedAt,
}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_channel_name")
                    .table(Channel::Table)
                    .col(Channel::Name)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_channel_name")
                    .table(Channel::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    Name,
}

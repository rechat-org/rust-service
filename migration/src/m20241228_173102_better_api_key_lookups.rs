use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add the prefix column
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("key_prefix"))
                            .string()
                            .not_null()
                            .default("") // Temporary default for existing rows
                    )
                    .to_owned(),
            )
            .await?;

        // Add composite index for fast lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_org_prefix")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::OrganizationId)
                    .col(Alias::new("key_prefix"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_api_keys_org_prefix")
                    .table(ApiKeys::Table)
                    .to_owned(),
            )
            .await?;

        // Then drop the column
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .drop_column(Alias::new("key_prefix"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ApiKeys {
    Table,
    OrganizationId,
}

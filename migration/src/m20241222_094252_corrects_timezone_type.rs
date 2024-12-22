use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Update organization_tiers table
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .modify_column(
                        ColumnDef::new(OrganizationTiers::CreatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .modify_column(
                        ColumnDef::new(OrganizationTiers::UpdatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        // Update api_keys table
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeys::LastUsedAt)
                            .timestamp()
                            .null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp()
                            .null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeys::CreatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeys::UpdatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        // Update api_key_usage table
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeyUsage::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeyUsage::CreatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeyUsage::UpdatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert organization_tiers table
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .modify_column(
                        ColumnDef::new(OrganizationTiers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .modify_column(
                        ColumnDef::new(OrganizationTiers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        // Revert api_keys table
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeys::LastUsedAt)
                            .timestamp_with_time_zone()
                            .null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        // Revert api_key_usage table
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeyUsage::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeyUsage::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .modify_column(
                        ColumnDef::new(ApiKeyUsage::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OrganizationTiers {
    Table,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiKeys {
    Table,
    LastUsedAt,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiKeyUsage {
    Table,
    CreatedAt,
    UpdatedAt,
}
use sea_orm_migration::prelude::*;
use crate::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create OrganizationTier enum
        manager
            .create_type(
                Type::create()
                    .as_enum(OrganizationTier::Table)
                    .values(vec![
                        OrganizationTier::Free,
                        OrganizationTier::Standard,
                        OrganizationTier::Pro,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create ApiKeyType enum
        manager
            .create_type(
                Type::create()
                    .as_enum(ApiKeyType::Table)
                    .values(vec![
                        ApiKeyType::Admin,
                        ApiKeyType::ReadWrite,
                        ApiKeyType::ReadOnly,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create organization_tiers table
        manager
            .create_table(
                Table::create()
                    .table(OrganizationTiers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrganizationTiers::OrganizationId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::Tier)
                            .enumeration(
                                OrganizationTier::Table,
                                vec![
                                    OrganizationTier::Free,
                                    OrganizationTier::Standard,
                                    OrganizationTier::Pro,
                                ]
                            )
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::DailyMessageLimit)
                            .integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-org_tiers-org_id")
                            .from(OrganizationTiers::Table, OrganizationTiers::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create api_keys table
        manager
            .create_table(
                Table::create()
                    .table(ApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKeys::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::OrganizationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ApiKeys::Name).string().not_null())
                    .col(ColumnDef::new(ApiKeys::Key).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(ApiKeys::KeyType)
                            .enumeration(
                                ApiKeyType::Table,
                                vec![
                                    ApiKeyType::Admin,
                                    ApiKeyType::ReadWrite,
                                    ApiKeyType::ReadOnly,
                                ]
                            )
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(ApiKeys::CreatedByUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::LastUsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-api_keys-org_id")
                            .from(ApiKeys::Table, ApiKeys::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-api_keys-user_id")
                            .from(ApiKeys::Table, ApiKeys::CreatedByUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // Create api_key_usage table for daily tracking
        manager
            .create_table(
                Table::create()
                    .table(ApiKeyUsage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKeyUsage::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ApiKeyUsage::ApiKeyId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeyUsage::Date)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeyUsage::MessageCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ApiKeyUsage::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeyUsage::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-api_key_usage-api_key_id")
                            .from(ApiKeyUsage::Table, ApiKeyUsage::ApiKeyId)
                            .to(ApiKeys::Table, ApiKeys::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on api_key_usage for date and api_key_id
        manager
            .create_index(
                Index::create()
                    .name("idx-api_key_usage-unique-date")
                    .table(ApiKeyUsage::Table)
                    .col(ApiKeyUsage::ApiKeyId)
                    .col(ApiKeyUsage::Date)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables
        manager
            .drop_table(Table::drop().table(ApiKeyUsage::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(OrganizationTiers::Table).to_owned())
            .await?;

        // Drop enum types
        manager
            .drop_type(Type::drop().name(ApiKeyType::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(OrganizationTier::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum OrganizationTier {
    Table,
    Free,
    Standard,
    Pro,
}

#[derive(DeriveIden)]
enum ApiKeyType {
    Table,
    Admin,
    ReadWrite,
    ReadOnly,
}

#[derive(DeriveIden)]
enum OrganizationTiers {
    Table,
    OrganizationId,
    Tier,
    DailyMessageLimit,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiKeys {
    Table,
    Id,
    OrganizationId,
    Name,
    Key,
    KeyType,
    CreatedByUserId,
    LastUsedAt,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiKeyUsage {
    Table,
    Id,
    ApiKeyId,
    Date,
    MessageCount,
    CreatedAt,
    UpdatedAt,
}
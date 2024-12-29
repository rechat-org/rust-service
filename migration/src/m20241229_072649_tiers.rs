use crate::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OrganizationTier {
    Table,
    Free,
    Basic,
    Pro,
    Enterprise,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // First, drop any columns using the enum type
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .drop_column(OrganizationTiers::Tier)
                    .to_owned(),
            )
            .await?;

        // Drop existing type first
        manager
            .drop_type(Type::drop().name(OrganizationTier::Table).to_owned())
            .await?;

        // Create new OrganizationTier enum
        manager
            .create_type(
                Type::create()
                    .as_enum(OrganizationTier::Table)
                    .values(vec![
                        OrganizationTier::Free,       // 0-5k
                        OrganizationTier::Basic,      // 5k-50k
                        OrganizationTier::Pro,        // 50k-500k
                        OrganizationTier::Enterprise, // 500k+
                    ])
                    .to_owned(),
            )
            .await?;

        // Create organization_tiers table for tracking limits and usage
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
                                    OrganizationTier::Basic,
                                    OrganizationTier::Pro,
                                    OrganizationTier::Enterprise,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::MonthlyRequestLimit)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::CurrentMonthUsage)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(OrganizationTiers::LastResetAt)
                            .timestamp_with_time_zone()
                            .not_null(),
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the organization_tiers table
        manager
            .drop_table(Table::drop().table(OrganizationTiers::Table).to_owned())
            .await?;

        // Drop the new enum type
        manager
            .drop_type(Type::drop().name(OrganizationTier::Table).to_owned())
            .await?;

        // Recreate original enum
        manager
            .create_type(
                Type::create()
                    .as_enum(OrganizationTier::Table)
                    .values(vec![
                        OrganizationTier::Free,
                        OrganizationTier::Pro,
                        OrganizationTier::Basic,
                    ])
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum OrganizationTiers {
    Table,
    OrganizationId,
    Tier,
    MonthlyRequestLimit,
    CurrentMonthUsage,
    LastResetAt,
    CreatedAt,
    UpdatedAt,
}

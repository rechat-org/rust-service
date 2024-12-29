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
        // Just alter the existing table to add new columns
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .add_column(
                        ColumnDef::new(OrganizationTiers::MonthlyRequestLimit)
                            .big_integer()
                            .not_null()
                            .default(5000),
                    )
                    .add_column(
                        ColumnDef::new(OrganizationTiers::CurrentMonthUsage)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(OrganizationTiers::LastResetAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the added columns
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .drop_column(OrganizationTiers::MonthlyRequestLimit)
                    .drop_column(OrganizationTiers::CurrentMonthUsage)
                    .drop_column(OrganizationTiers::LastResetAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum OrganizationTiers {
    Table,
    Tier,
    MonthlyRequestLimit,
    CurrentMonthUsage,
    LastResetAt,
    CreatedAt,
    UpdatedAt,
}

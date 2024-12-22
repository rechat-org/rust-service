use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the daily_message_limit column
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .drop_column(OrganizationTiers::DailyMessageLimit)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add back the daily_message_limit column
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationTiers::Table)
                    .add_column(
                        ColumnDef::new(OrganizationTiers::DailyMessageLimit)
                            .integer()
                            .not_null()
                            .default(5000)
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
    DailyMessageLimit,
}
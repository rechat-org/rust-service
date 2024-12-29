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
                            .default(OrganizationTier::Free.to_string())
                            .not_null(),
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
                    .drop_column(OrganizationTiers::Tier)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum OrganizationTiers {
    Table,
    Tier,
}

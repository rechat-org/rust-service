use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add Stripe-related columns to organizations
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .add_column(
                        ColumnDef::new(Organizations::StripeCustomerId)
                            .string()
                            .null(), // Nullable because it might be set after creation
                    )
                    .add_column(
                        ColumnDef::new(Organizations::StripeSubscriptionId)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .drop_column(Organizations::StripeCustomerId)
                    .drop_column(Organizations::StripeSubscriptionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    StripeCustomerId,
    StripeSubscriptionId,
}

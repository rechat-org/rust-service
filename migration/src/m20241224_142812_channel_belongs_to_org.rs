use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .add_column(
                        ColumnDef::new(Channel::OrganizationId)
                            .uuid()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-channel-org_id")
                    .from(Channel::Table, Channel::OrganizationId)
                    .to(Organizations::Table, Organizations::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add index for organization_id
        manager
            .create_index(
                Index::create()
                    .name("idx-channels-org_id")
                    .table(Channel::Table)
                    .col(Channel::OrganizationId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-channels-org_id")
                    .table(Channel::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .drop_column(Channel::OrganizationId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    OrganizationId,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}
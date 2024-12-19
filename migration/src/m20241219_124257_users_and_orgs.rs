use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Create organizations table
        manager
            .create_table(
                Table::create()
                    .table(Organizations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Organizations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Organizations::Name).string().not_null())
                    .col(
                        ColumnDef::new(Organizations::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Organizations::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Organizations::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create organization_members table
        manager
            .create_table(
                Table::create()
                    .table(OrganizationMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrganizationMembers::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::OrganizationId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::Role)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::CreatedAt)
                            .timestamp().not_null(),
                    )
                    .col(
                        ColumnDef::new(OrganizationMembers::UpdatedAt)
                            .timestamp().not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(OrganizationMembers::UserId)
                            .col(OrganizationMembers::OrganizationId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-org_members-user_id")
                            .from(OrganizationMembers::Table, OrganizationMembers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-org_members-org_id")
                            .from(
                                OrganizationMembers::Table,
                                OrganizationMembers::OrganizationId,
                            )
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-organizations-slug")
                    .table(Organizations::Table)
                    .col(Organizations::Slug)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrganizationMembers::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
    Name,
    Slug,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum OrganizationMembers {
    Table,
    UserId,
    OrganizationId,
    Role,
    CreatedAt,
    UpdatedAt,
}

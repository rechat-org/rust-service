use sea_orm_migration::prelude::*;
use crate::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // First create the role enum type
        manager
            .create_type(
                Type::create()
                    .as_enum(OrganizationRole::Table)
                    .values(vec![
                        OrganizationRole::Owner,
                        OrganizationRole::Admin,
                        OrganizationRole::Developer,
                        OrganizationRole::ReadOnly,
                    ])
                    .to_owned(),
            )
            .await?;

        // Drop the existing slug index
        manager
            .drop_index(
                Index::drop()
                    .name("idx-organizations-slug")
                    .table(Organizations::Table)
                    .to_owned(),
            )
            .await?;

        // Remove the slug column from organizations
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .drop_column(Organizations::Slug)
                    .to_owned(),
            )
            .await?;

        // For the role column, we need to:
        // 1. Add a new column of type organization_role
        // 2. Convert existing data
        // 3. Drop the old column
        // 4. Rename the new column

        // Add new column
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationMembers::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("role_new"))
                            .enumeration(
                                OrganizationRole::Table,
                                vec![
                                    OrganizationRole::Owner,
                                    OrganizationRole::Admin,
                                    OrganizationRole::Developer,
                                    OrganizationRole::ReadOnly,
                                ]
                            )
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Convert existing data using raw SQL
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE organization_members 
                SET role_new = role::organization_role;
                "#
            )
            .await?;

        // Drop old column
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationMembers::Table)
                    .drop_column(OrganizationMembers::Role)
                    .to_owned(),
            )
            .await?;

        // Rename new column
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationMembers::Table)
                    .rename_column(Alias::new("role_new"), OrganizationMembers::Role)
                    .to_owned(),
            )
            .await?;

        // Make the column not null
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationMembers::Table)
                    .modify_column(
                        ColumnDef::new(OrganizationMembers::Role)
                            .enumeration(
                                OrganizationRole::Table,
                                vec![
                                    OrganizationRole::Owner,
                                    OrganizationRole::Admin,
                                    OrganizationRole::Developer,
                                    OrganizationRole::ReadOnly,
                                ]
                            )
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Convert role column back to string
        manager
            .alter_table(
                Table::alter()
                    .table(OrganizationMembers::Table)
                    .modify_column(
                        ColumnDef::new(OrganizationMembers::Role)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add slug column back to organizations
        manager
            .alter_table(
                Table::alter()
                    .table(Organizations::Table)
                    .add_column(
                        ColumnDef::new(Organizations::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // Recreate the slug index
        manager
            .create_index(
                Index::create()
                    .name("idx-organizations-slug")
                    .table(Organizations::Table)
                    .col(Organizations::Slug)
                    .to_owned(),
            )
            .await?;

        // Drop the role enum type
        manager
            .drop_type(Type::drop().name(OrganizationRole::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Slug,
}

#[derive(DeriveIden)]
enum OrganizationMembers {
    Table,
    Role,
}

#[derive(DeriveIden)]
enum OrganizationRole {
    Table,
    Owner,
    Admin,
    Developer,
    ReadOnly,
}
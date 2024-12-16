use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Channel::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Channel::Name).string().not_null())
                    .col(ColumnDef::new(Channel::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Channel::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Participant::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Participant::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Participant::Name).string().not_null())
                    .col(
                        ColumnDef::new(Participant::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Participant::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ChannelParticipant::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChannelParticipant::ChannelId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelParticipant::ParticipantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelParticipant::CreatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(ChannelParticipant::UpdatedAt)
                            .timestamp()
                            .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_channel_participant")
                            .col(ChannelParticipant::ChannelId)
                            .col(ChannelParticipant::ParticipantId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChannelParticipant::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Participant::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Participant {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ChannelParticipant {
    Table,
    ChannelId,
    ParticipantId,
    CreatedAt,
    UpdatedAt,
}

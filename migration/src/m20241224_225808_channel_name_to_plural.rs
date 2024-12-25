use crate::m20241216_081358_channel_participant_relationship::Participant;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop existing tables
        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await?;

        // Create channels table
        manager
            .create_table(
                Table::create()
                    .table(Channels::Table)
                    .col(ColumnDef::new(Channels::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Channels::Name).string().not_null())
                    .col(ColumnDef::new(Channels::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Channels::UpdatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Channels::OrganizationId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_channels_organization")
                            .from(Channels::Table, Channels::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create messages table
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .col(ColumnDef::new(Messages::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Messages::Content).string().not_null())
                    .col(ColumnDef::new(Messages::ChannelId).uuid().not_null())
                    .col(ColumnDef::new(Messages::ParticipantId).uuid().not_null())
                    .col(ColumnDef::new(Messages::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Messages::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_messages_channel")
                            .from(Messages::Table, Messages::ChannelId)
                            .to(Channels::Table, Channels::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_participant_id")
                            .from(Messages::Table, Messages::ParticipantId)
                            .to(Participant::Table, Participant::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .table(Messages::Table)
                    .name("idx_messages_channel")
                    .col(Messages::ChannelId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Messages::Table)
                    .name("idx_messages_participant")
                    .col(Messages::ParticipantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Channels::Table).to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .col(ColumnDef::new(Channel::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Channel::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Channel::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Channel::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
    OrganizationId,
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
enum Messages {
    Table,
    Id,
    Content,
    CreatedAt,
    UpdatedAt,
    ParticipantId,
    ChannelId,
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
}

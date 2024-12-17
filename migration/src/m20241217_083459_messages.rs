use crate::m20241216_081358_channel_participant_relationship::{
    Channel, ChannelParticipant, Participant,
};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Messages::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Messages::Content).string().not_null())
                    .col(ColumnDef::new(Messages::ChannelId).uuid().not_null())
                    .col(ColumnDef::new(Messages::ParticipantId).uuid().not_null())
                    .col(ColumnDef::new(Messages::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Messages::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_channel_id")
                            .from(Messages::Table, Messages::ChannelId)
                            .to(Channel::Table, Channel::Id),
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

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_channel")
                    .table(Messages::Table)
                    .col(Messages::ChannelId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_messages_participant")
                    .table(Messages::Table)
                    .col(Messages::ParticipantId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_messages_channel")
                    .table(Messages::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_messages_participant")
                    .table(Messages::Table)
                    .to_owned(),
            )
            .await?;

        // Then drop the table
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    Id,
    Content,
    ChannelId,
    ParticipantId,
    CreatedAt,
    UpdatedAt,
}

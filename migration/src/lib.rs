pub use sea_orm_migration::prelude::*;

mod m20241216_081358_channel_participant_relationship;
mod m20241217_083459_messages;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241216_081358_channel_participant_relationship::Migration),
            Box::new(m20241217_083459_messages::Migration),
        ]
    }
}

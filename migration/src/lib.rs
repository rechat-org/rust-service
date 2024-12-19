pub use sea_orm_migration::prelude::*;

mod m20241216_081358_channel_participant_relationship;
mod m20241217_083459_messages;
mod m20241217_115248_channel_name_is_unique;
mod m20241217_115805_channel_name_index;
mod m20241219_124257_users_and_orgs;
mod m20241219_210908_fixes_organization_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241216_081358_channel_participant_relationship::Migration),
            Box::new(m20241217_083459_messages::Migration),
            Box::new(m20241217_115248_channel_name_is_unique::Migration),
            Box::new(m20241217_115805_channel_name_index::Migration),
            Box::new(m20241219_124257_users_and_orgs::Migration),
            Box::new(m20241219_210908_fixes_organization_table::Migration),
        ]
    }
}

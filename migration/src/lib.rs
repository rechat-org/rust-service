pub use sea_orm_migration::prelude::*;

mod m20241216_081358_channel_participant_relationship;
mod m20241217_083459_messages;
mod m20241217_115248_channel_name_is_unique;
mod m20241217_115805_channel_name_index;
mod m20241219_124257_users_and_orgs;
mod m20241219_210908_fixes_organization_table;
mod m20241222_090954_api_key_management;
mod m20241222_093435_remove_daily;
mod m20241222_094252_corrects_timezone_type;
mod m20241224_092916_stripe_int_the_mix;
mod m20241224_142812_channel_belongs_to_org;
mod m20241224_225808_channel_name_to_plural;
mod m20241228_173102_better_api_key_lookups;
mod m20241229_072649_tiers;
mod m20241229_094545_si_for_stripe;
mod m20241229_102530_adds_back_tier_col;
mod m20241229_103359_adds_tier;

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
            Box::new(m20241222_090954_api_key_management::Migration),
            Box::new(m20241222_093435_remove_daily::Migration),
            Box::new(m20241222_094252_corrects_timezone_type::Migration),
            Box::new(m20241224_092916_stripe_int_the_mix::Migration),
            Box::new(m20241224_142812_channel_belongs_to_org::Migration),
            Box::new(m20241224_225808_channel_name_to_plural::Migration),
            Box::new(m20241228_173102_better_api_key_lookups::Migration),
            Box::new(m20241229_072649_tiers::Migration),
            Box::new(m20241229_094545_si_for_stripe::Migration),
            Box::new(m20241229_102530_adds_back_tier_col::Migration),
            Box::new(m20241229_103359_adds_tier::Migration),
        ]
    }
}

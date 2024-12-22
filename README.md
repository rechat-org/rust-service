# chat-service


# Database Migrations

We use SeaORM migrations to manage our database schema. Here's how to work with migrations:

### Creating a New Migration

1. Generate a new migration file:
   ```bash
   cd migrations
   cargo run -- generate MIGRATION_NAME
   # Example: cargo run -- generate api_key_management
   ```

2. The migration will be created in `./migrations/src/mYYYYMMDD_HHMMSS_MIGRATION_NAME.rs`
3. It will also be automatically added to `./src/lib.rs`

### Running Migrations

To apply all pending migrations:
```bash
cargo run
```

### Generating Entities

After creating new tables, generate the SeaORM entities:
```bash
cd ../
sea-orm-cli generate entity --with-serde both -o src/entities
```

### Migration Example

Here's a basic structure of a migration file:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Your up migration code here
        manager
            .create_table(...)
            .await?;
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Your down migration code here
        manager
            .drop_table(...)
            .await?;
            
        Ok(())
    }
}
```

### Best Practices

1. Always implement both `up` and `down` migrations
2. Test migrations locally before pushing
3. Make migrations idempotent when possible

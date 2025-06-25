# Chat Service

A Rust-based chat service API built with Axum, SeaORM, and PostgreSQL.

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- Redis server
- Environment variables configured (see `.env.example`)

## Environment Setup

1. Copy the environment example file:
   ```bash
   cp .env.example .env
   ```

2. Update the `.env` file with your database and service configurations:
   - `DATABASE_URL`: PostgreSQL connection string
   - `REDIS_URL`: Redis connection string
   - `STRIPE_SECRET_KEY`: Stripe API secret key
   - `STRIPE_PRICE_ID`: Stripe price ID for subscriptions
   - `JWT_SECRET`: Secret key for JWT token generation

## Database Migrations

We use SeaORM migrations to manage our database schema. Here's how to work with migrations:

### Creating a New Migration

1. Generate a new migration file:
   ```bash
   cd migration
   cargo run -- generate MIGRATION_NAME
   # Example: cargo run -- generate api_key_management
   ```

2. The migration will be created in `./migration/src/mYYYYMMDD_HHMMSS_MIGRATION_NAME.rs`
3. It will also be automatically added to `./migration/src/lib.rs`

### Running Migrations

To apply all pending migrations:
```bash
cd migration
cargo run
```

### Generating Entities

After creating new tables, generate the SeaORM entities:
```bash
sea-orm-cli generate entity --with-serde both -o src/entities
```

### Migration Commands

For more detailed migration operations, see the migration directory README or use these commands:

```bash
cd migration

# Check migration status
cargo run -- status

# Apply all pending migrations
cargo run -- up

# Apply first N pending migrations
cargo run -- up -n 5

# Rollback last applied migration
cargo run -- down

# Rollback last N applied migrations
cargo run -- down -n 3

# Drop all tables and reapply all migrations
cargo run -- fresh

# Rollback all migrations and reapply them
cargo run -- refresh

# Rollback all applied migrations
cargo run -- reset
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
4. Handle foreign key constraints properly when dropping tables

## Running the Service

### Local Development

1. Start the required services using Docker Compose:
   ```bash
   docker-compose up -d
   ```

2. Apply database migrations:
   ```bash
   cd migration
   cargo run
   ```

3. Run the service:
   ```bash
   cargo run
   ```

The service will be available at `http://localhost:3001` by default.

### Production Deployment

The service is configured for deployment on DigitalOcean Kubernetes using:
- DigitalOcean Container Registry for Docker images
- DigitalOcean Managed PostgreSQL database
- DigitalOcean Managed Redis (Valkey) cluster
- Helm charts for Kubernetes deployment

## API Documentation

The service provides a REST API for chat functionality including:
- User and organization management
- Channel creation and management
- Message sending and retrieval
- API key authentication
- Stripe integration for billing

## Helm Chart Usage

## How to upgrade locally

1. cd into the `helm` directory
   ```
   cd helm
   ```

2. Run the following command to upgrade the helm chart
   ```
   helm upgrade chat-service ./chat-service
   ```


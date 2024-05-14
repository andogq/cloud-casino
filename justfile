set dotenv-load

default:
    @just --list

# Run `cargo watch`, which will restart the application automatically as files change.
dev:
    cargo watch -c -x run -i static -i data

# Build the docker image
build:
    docker build -t cloud-casino .

# Create a new migration
new-migration name:
    just sqlx migrate add {{ name }}

# Apply a migration and revert it to test the up and down scripts.
test-migration:
    # Run the migration to test the up script
    just sqlx migrate run

    # Revert the migration to test the down script
    just sqlx migrate revert

# Undo the latest migration and re-run it
redo-migration:
    just sqlx migrate revert
    just sqlx migrate run

# Apply and run any pending migrations
apply-migration:
    just sqlx migrate run

# Run sqlx commands
sqlx +command:
    cargo sqlx {{ command }}

# Open the database with sqlite3
db:
    sqlite3 -header -box $DATABASE_PATH

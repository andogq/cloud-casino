set dotenv-load := true

default:
    @just --list

# Run `cargo watch`, which will restart the application automatically as files change.
dev:
    cargo watch -c -x run -i static

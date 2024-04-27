set dotenv-filename := ".dev.env"

default:
    @just --list

# Run `cargo watch`, which will restart the application automatically as files change.
dev:
    cargo watch -c -x run -i static

build:
    #!/bin/bash
    source .env # Use the production environment variables

    docker build -t cloud-casino .

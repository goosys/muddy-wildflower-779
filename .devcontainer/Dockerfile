FROM mcr.microsoft.com/devcontainers/rust:1

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
     && cargo install loco-cli cargo-insta cargo-cache cargo-shuttle \
     && chown -R vscode /usr/local/cargo

COPY .env /.env

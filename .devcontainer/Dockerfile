FROM mcr.microsoft.com/devcontainers/rust:1

COPY rust-toolchain.toml /tmp/worker-toolchain/rust-toolchain.toml
RUN cd /tmp/worker-toolchain \
    && cargo install worker-build \
        --git https://github.com/cloudflare/workers-rs \
        --rev b57ba6ef8198c65499c2f92b1845cc2412dd6e8c --locked \
    && chown -R vscode /usr/local/cargo

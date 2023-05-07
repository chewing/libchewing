# MSRV: 1.61
# Debian (sid, bookworm): 1.63.0
# Ubuntu LTS: 1.61.0
ARG RUST_VERSION=1.61.0

FROM rust:${RUST_VERSION} AS rust-env

# Final stage
# https://github.com/devcontainers/images/tree/main/src/base-debian
FROM mcr.microsoft.com/devcontainers/base:debian

ARG RUST_VERSION
ARG USER=vscode

# === Basic utility
RUN DEBIAN_FRONTEND=noninteractive \
    set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    git \
    make \
    cmake \
    libsqlite3-dev \
    ; \
    rm -rf /var/lib/apt/lists/*

USER ${USER}
# ===========
#    Rust
# ===========
ENV RUSTUP_HOME=/home/${USER}/.rustup \
    CARGO_HOME=/home/${USER}/.cargo \
    PATH=/home/${USER}/.cargo/bin:$PATH \
    RUST_VERSION=${RUST_VERSION}

# Use sparse protocol once MSRV reaches 1.68.0
# Ref: Sparse index protocol: https://blog.rust-lang.org/inside-rust/2023/01/30/cargo-sparse-protocol.html
#
# ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

COPY --chown=${USER} --from=rust-env /usr/local/cargo /home/${USER}/.cargo
COPY --chown=${USER} --from=rust-env /usr/local/rustup /home/${USER}/.rustup

RUN set -eux; \
    rustup component add \
    clippy \
    rustfmt \
    ;

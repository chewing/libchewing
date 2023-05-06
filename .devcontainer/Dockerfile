ARG RUST_VERSION=1.68.2
ARG CMAKE_VERSION=3.26.3

FROM rust:${RUST_VERSION} AS rust-env

FROM debian:11-slim as build-base
RUN DEBIAN_FRONTEND=noninteractive \
    set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    tar \
    ; \
    rm -rf /var/lib/apt/lists/*

FROM build-base as cmake-env
ARG CMAKE_VERSION
RUN set -eux; \
    curl -LO https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz; \
    tar -xzf cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz; \
    mv cmake-${CMAKE_VERSION}-linux-x86_64 /usr/local/cmake;

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

# Sparse index protocol: https://blog.rust-lang.org/inside-rust/2023/01/30/cargo-sparse-protocol.html
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

COPY --chown=${USER} --from=rust-env /usr/local/cargo /home/${USER}/.cargo
COPY --chown=${USER} --from=rust-env /usr/local/rustup /home/${USER}/.rustup

RUN set -eux; \
    rustup component add clippy rustfmt rust-analyzer;


# ===========
#    Cmake
# ===========
COPY --chown=${USER} --from=cmake-env /usr/local/cmake /home/${USER}/usr/local/cmake
ENV PATH=/home/${USER}/usr/local/cmake/bin:$PATH
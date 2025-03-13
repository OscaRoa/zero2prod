FROM rust:1.85.0-slim-bookworm as builder
LABEL authors="Oscar Roa"

RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    lld \
    postgresql

WORKDIR /app

RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

# Temporary main and lib used for caching dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && touch src/lib.rs && echo "fn main () {}" > src/main.rs
RUN cargo build

COPY . .
RUN cargo build

EXPOSE 8000
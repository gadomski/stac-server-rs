FROM rust:1-slim-buster AS chef
WORKDIR /app
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get install -y build-essential libssl-dev pkg-config git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get install -y libssl1.1 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/stac-server /usr/local/bin
COPY data /app/data
COPY example-config.toml /app/example-config.toml
CMD [ "/usr/local/bin/stac-server" ]

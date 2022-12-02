FROM rust:1-slim-buster as base

FROM base as builder

RUN cargo new /usr/src/app
WORKDIR /usr/src/app
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir .cargo
RUN cargo vendor > .cargo/config.toml

COPY ./src src
RUN cargo build --release
RUN cargo install --path . --verbose

CMD [ "stac-server" ]

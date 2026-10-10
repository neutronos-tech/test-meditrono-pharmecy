FROM rust:1.94.0-bookworm AS build

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

ENV CARGO_BUILD_JOBS=1
RUN cargo build --locked --release -p server

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /src/target/release/server /app/server

ENV ADDR=0.0.0.0:4100
EXPOSE 4100
CMD ["/app/server"]

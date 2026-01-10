FROM rustlang/rust:nightly-slim@sha256:f731f594c6fedd4713b2c3b0bc8eb9434d0f6abe2a9dfd1a96dce88aaf6eb169 AS chef
RUN apt update && apt install -y curl pkg-config libssl-dev
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN mkdir ~/.cargo && \
    printf '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-Zshare-generics=y", "-Zthreads=8"]' > ~/.cargo/Config.toml

COPY --from=planner /recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo +nightly build --release

FROM node:slim@sha256:b20f82c2332a22609edb43cdfd6b16cbb199a6e6fd5cf8d5ef85b43faab43144 AS runtime
RUN apt update && apt install -y curl libcurl4

COPY --from=builder /target/release/mrt-flightradar2 .
RUN mkdir ats

EXPOSE 3000
CMD ["./mrt-flightradar2"]

FROM rustlang/rust:nightly-slim@sha256:8f7fa066c1dca95b5c66ecb0781cb29d761ceb36daf7af096bb42d6f722a5fab AS chef
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

FROM node:slim@sha256:9d346b36433145de8bde85fb11f37820ae7b3fcf0b0771d0fbcfa01c79607909 AS runtime
RUN apt update && apt install -y curl libcurl4

COPY --from=builder /target/release/mrt-flightradar2 .
RUN mkdir ats

EXPOSE 3000
CMD ["./mrt-flightradar2"]

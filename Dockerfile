FROM rustlang/rust:nightly-slim@sha256:9b11303753d9e5d865451b81c72245f2a19f702a6f2ad6074cc0786d4f5d1d94 AS chef
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

FROM node:slim@sha256:d9dab42dc0e575d6e959b7b3d6962fd35cb4ef668e6ad4b4cda135488c504bbe AS runtime
RUN apt update && apt install -y curl libcurl4

COPY --from=builder /target/release/mrt-flightradar2 .
RUN mkdir ats

EXPOSE 3000
CMD ["./mrt-flightradar2"]

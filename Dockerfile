FROM rustlang/rust:nightly-slim@sha256:cdfe0059426240c64e096dbad943fc8d3c6bb9b40131c8523f063bc5e72f58c5 AS chef
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

FROM node:slim@sha256:7363aaa1daa5b06a01711960c10c58f1bc1c5eebf94fd52df62ed220e6771624 AS runtime
RUN apt update && apt install -y curl libcurl4

COPY --from=builder /target/release/mrt-flightradar2 .
RUN mkdir ats

EXPOSE 3000
CMD ["./mrt-flightradar2"]

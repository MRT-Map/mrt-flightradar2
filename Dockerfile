FROM rustlang/rust:nightly-slim@sha256:e28354dd5511f4231317d30adda0a3386ab8d9a0e245a4eb87e775b2c219304b AS chef
WORKDIR /app
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
# renovate: deb depName=curl
ENV CURL_VERSION="8.14.1-2+deb13u2"
# renovate: deb depName=pkg-config
ENV PKG_CONFIG_VERSION="1.8.1-4"
# renovate: deb depName=libssl-dev
ENV LIBSSL_VERSION="3.5.4-1~deb13u2"
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl="${CURL_VERSION}" \
    pkg-config="${PKG_CONFIG_VERSION}" \
    libssl-dev="${LIBSSL_VERSION}" \
    && rm -r /var/lib/apt/lists/* && apt-get clean
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN mkdir ~/.cargo && \
    printf '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-Zshare-generics=y", "-Zthreads=8"]' > ~/.cargo/Config.toml

COPY --from=planner /app/recipe.json recipe.json

FROM builder AS check
RUN cargo chef cook --release --check --recipe-path recipe.json
COPY . .
RUN cargo +nightly check --release

FROM builder AS build
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo +nightly build --release


FROM node:trixie-slim@sha256:f8e300c21d41d23cf53ce0fe60478d491039f01a55107966d690574d1692587c AS runtime
WORKDIR /app
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]

# renovate: deb depName=curl
ENV CURL_VERSION="8.14.1-2+deb13u2"
# renovate: deb depName=libcurl4t64
ENV LIBCURL_VERSION="8.14.1-2+deb13u2"

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl="${CURL_VERSION}" \
    libcurl4t64="${LIBCURL_VERSION}" \
    && rm -r /var/lib/apt/lists/* && apt-get clean

COPY --from=build /app/target/release/mrt-flightradar2 .
RUN mkdir ats

EXPOSE 3000
CMD ["./mrt-flightradar2"]

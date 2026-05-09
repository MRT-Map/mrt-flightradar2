FROM rustlang/rust:nightly-slim@sha256:8cc355a1eed9a60a7b555d5e9c31e8886033fccabb9d6c1f6886080f970f2a87 AS chef
WORKDIR /app
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
# renovate: datasource=deb depName=curl
ENV CURL_VERSION="8.14.1-2+deb13u2"
# renovate: datasource=deb depName=pkg-config
ENV PKG_CONFIG_VERSION="1.8.1-4"
# renovate: datasource=deb depName=libssl-dev
ENV LIBSSL_VERSION="3.5.5-1~deb13u2"
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


FROM node:trixie-slim@sha256:424cafd2a035ed2b2d74acc3142b68b426fb62a47742c80a75e7117db02d6b30 AS runtime
WORKDIR /app
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]

# renovate: datasource=deb depName=curl
ENV CURL_VERSION="8.14.1-2+deb13u2"
# renovate: datasource=deb depName=libcurl4t64
ENV LIBCURL_VERSION="8.14.1-2+deb13u2"

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl="${CURL_VERSION}" \
    libcurl4t64="${LIBCURL_VERSION}" \
    && rm -r /var/lib/apt/lists/* && apt-get clean

COPY --from=build /app/target/release/mrt-flightradar2 .
RUN mkdir ats

EXPOSE 3000
CMD ["./mrt-flightradar2"]

FROM rust AS build
RUN rustup toolchain install nightly
COPY . .
RUN cargo +nightly build --release


FROM rust:slim
RUN apt update && apt install -y nodejs npm curl libcurl4

COPY --from=build /target/release/mrt-flightradar2 .
EXPOSE 3000
CMD ["./mrt-flightradar2"]

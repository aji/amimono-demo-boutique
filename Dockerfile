FROM rust:1-slim-trixie AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
WORKDIR /app
COPY --from=build /app/target/release/amimono-demo-boutique /app/amimono-demo-boutique
COPY --from=build /app/static /app/static
ENTRYPOINT ["/app/amimono-demo-boutique"]
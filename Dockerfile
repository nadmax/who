FROM rust:1.95-alpine AS chef
RUN apk add --no-cache musl-dev pkgconfig curl && \
    cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY ./ ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./ ./
RUN cargo build --release

FROM alpine:3.23 AS runtime
RUN apk add --no-cache ca-certificates
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app
COPY --from=builder /app/target/release/who ./
USER app
EXPOSE 8080
ENTRYPOINT ["./who"]
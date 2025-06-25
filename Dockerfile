FROM rust:1.87-alpine as builder

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-libs-static pkgconfig

COPY Cargo.toml Cargo.lock ./
COPY service/Cargo.toml ./service/
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --package service
RUN rm -rf src service/Cargo.toml

COPY . .

RUN cargo build --release --package service

FROM alpine:latest

RUN apk add --no-cache openssl

WORKDIR /app

COPY --from=builder /app/target/release/service /usr/local/bin/service

CMD ["service"]

FROM rust:1.87-alpine as builder

WORKDIR /app

RUN apk add --no-cache openssl-dev musl-dev openssl-libs-static pkgconfig

COPY Cargo.toml Cargo.lock ./
COPY service/Cargo.toml service/Cargo.toml
COPY macros/Cargo.toml macros/Cargo.toml

RUN mkdir service/src macros/src
RUN echo "fn main() {}" > service/src/main.rs
RUN echo "" > macros/src/lib.rs

RUN cargo build --release -p service || true

COPY . .

RUN cargo build --release -p service

FROM alpine:latest

RUN apk add --no-cache openssl

WORKDIR /app

COPY --from=builder /app/target/release/service /usr/local/bin/service

CMD ["service"]
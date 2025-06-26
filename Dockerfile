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

ARG STRIPE__SECRET_KEY
ARG STRIPE__WEBHOOK_SECRET
ARG WEBHOOK__URL

ENV STRIPE__SECRET_KEY=${STRIPE__SECRET_KEY}
ENV STRIPE__WEBHOOK_SECRET=${STRIPE__WEBHOOK_SECRET}
ENV WEBHOOK__URL=${WEBHOOK__URL}

RUN apk add --no-cache openssl

WORKDIR /app

COPY --from=builder /app/target/release/service /service

CMD ["/service"]
FROM rust:1.87-alpine

WORKDIR /app
COPY . .

RUN apk add --no-cache musl-dev \
	openssl-dev \
	pkgconfig

RUN cargo install --path ./service
RUN ["service"]
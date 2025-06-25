FROM rust:1.87-alpine

WORKDIR /app
COPY . .

RUN cargo install --path ./service
RUN ["service"]
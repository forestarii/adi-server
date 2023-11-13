# Use a Rust base image
FROM rust:latest as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

# Create a new, smaller image to reduce the final image size
FROM debian:buster-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/adi-server /usr/src/app/adi-server

CMD ["./adi-server"]

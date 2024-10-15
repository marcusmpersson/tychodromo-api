FROM rust:1.81 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

FROM debian:buster-slim as runner
COPY --from=builder /usr/local/cargo/bin/rust-rocket-app /usr/local/bin/rust-rocket-app
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["tychodromo-api"]

# Builder stage
FROM rust:1.81 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

# Runner stage
FROM ubuntu:22.04 as runner
RUN apt-get update && apt-get install -y openssl ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/tychodromo-api /usr/local/bin/tychodromo-api
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["tychodromo-api"]

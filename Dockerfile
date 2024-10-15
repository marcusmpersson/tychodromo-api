FROM rust:1.81 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .


FROM debian:buster-slim as runner
COPY --from=builder /usr/local/cargo/bin/tychodromo-api /usr/local/bin/tychodromo-api
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["tychodromo-api"]

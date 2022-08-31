FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/payment-engine
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/payment-engine /usr/local/bin/payment-engine
ENTRYPOINT ["payment-engine"]
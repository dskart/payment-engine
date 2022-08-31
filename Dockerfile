FROM rustlang/rust:nightly

WORKDIR /rust/src/payment-engine
COPY . .

RUN cargo install --path .

CMD ["payment-engine"]
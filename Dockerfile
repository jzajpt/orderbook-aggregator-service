FROM rust:1.50

WORKDIR /usr/src/orderbook-aggregator
COPY . .

RUN rustup component add rustfmt
RUN cargo install --path .

ENV PAIR=ethbtc

CMD ["aggregator-server"]

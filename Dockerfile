FROM rust:1.50

WORKDIR /usr/src/keyrock-challenge
COPY . .

RUN cargo install --path .

CMD ["keyrock-challenge"]


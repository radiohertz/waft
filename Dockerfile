FROM rust:1.86

WORKDIR /usr/src/waft
COPY . .

RUN cargo install --locked --path .

CMD ["waft"]

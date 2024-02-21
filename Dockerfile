FROM rust:1.76-alpine

WORKDIR /usr/src/dateframe
COPY . .

RUN cargo install --path .

CMD ["dateframe", "/var/data"]
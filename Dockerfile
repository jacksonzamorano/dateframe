FROM rust:1.76

WORKDIR /usr/src/dateframe
COPY . .

RUN cargo install --path .

CMD ["dateframe", "/var/data"]
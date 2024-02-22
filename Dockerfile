FROM rust:1.76-alpine

WORKDIR /usr/src/dateframe
COPY . .

RUN apk add --no-cache tzdata
RUN cargo install --path .

CMD ["dateframe", "/var/data"]
FROM rust:1.63 as builder

COPY ./sources.list /etc/apt/sources.list
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 3B4FE6ACC0B21F32

RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/phone_data
COPY . .
COPY ./config.toml  /usr/local/cargo

RUN RUSTFLAGS=-Clinker=musl-gcc cargo install -- --release -target=x86_64-unknown-linux-musl


FROM alpine:latest

COPY --from=builder /usr/local/cargo/bin/phone_data /usr/local/bin/phone_data
COPY --from=builder /usr/src/phone_data/phone.dat /usr/src/phone.dat
CMD ["phone_data"]
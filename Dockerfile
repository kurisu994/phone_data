FROM rust:1.70 as builder

WORKDIR /usr/local/src/
COPY . ./

RUN cargo install --path .

FROM ubuntu:latest
ENV TZ=Asia/Shanghai

COPY --from=builder /usr/local/cargo/bin/phone_data /usr/local/bin/phone_data
COPY --from=builder /usr/local/src/phone.dat /usr/local/bin/phone.dat

WORKDIR /usr/local/bin/

CMD ["./phone_data"]
FROM rust:1.91-slim AS builder
ARG TARGETARCH
ARG TARGETVARIANT

WORKDIR /usr/local/src
COPY . .

RUN mkdir -p benches && \
    echo 'fn main() {}' > benches/lookup_performance.rs && \
    echo 'fn main() {}' > benches/algorithm_comparison.rs

RUN apt-get update && apt-get install -y --no-install-recommends musl-tools binutils && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    if [ "$TARGETARCH" = "amd64" ]; then TARGET=x86_64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm64" ]; then TARGET=aarch64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm" ] && [ "$TARGETVARIANT" = "v7" ]; then TARGET=armv7-unknown-linux-musleabihf; \
    else echo "unsupported arch: ${TARGETARCH}${TARGETVARIANT}"; exit 1; fi; \
    rustup target add "$TARGET"; \
    cargo build --release --target "$TARGET"; \
    strip "target/$TARGET/release/phone_data"; \
    cp "target/$TARGET/release/phone_data" "target/release/phone_data"

FROM alpine:3.22
ENV TZ=Asia/Shanghai
COPY --from=builder /usr/local/src/target/release/phone_data /usr/local/bin/phone_data
COPY --from=builder /usr/local/src/phone.dat /usr/local/bin/phone.dat
WORKDIR /usr/local/bin
USER 65532
CMD ["./phone_data"]
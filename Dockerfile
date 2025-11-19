FROM rust:1.91-alpine AS builder
ARG TARGETARCH
ARG TARGETVARIANT

# 安装构建依赖（actix-web 需要 openssl-dev 和 pkgconfig）
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    gcc

# 创建空项目以缓存依赖
RUN USER=root cargo new --bin phone_data
WORKDIR /phone_data
COPY ./Cargo.toml ./Cargo.toml

# 创建空的 benchmark 文件以满足 Cargo.toml 的要求
RUN mkdir -p benches && \
    touch benches/lookup_performance.rs && \
    touch benches/algorithm_comparison.rs

# 预先构建依赖以利用 Docker 缓存层
RUN set -eux; \
    if [ "$TARGETARCH" = "amd64" ]; then TARGET=x86_64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm64" ]; then TARGET=aarch64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm" ] && [ "$TARGETVARIANT" = "v7" ]; then TARGET=armv7-unknown-linux-musleabihf; \
    else TARGET=x86_64-unknown-linux-musl; fi; \
    rustup target add "$TARGET"; \
    cargo build --release --target "$TARGET"

# 删除临时文件并复制实际源码
RUN rm src/*.rs
COPY ./src ./src
COPY ./phone.dat ./phone.dat

# 删除旧的编译产物并重新编译
RUN set -eux; \
    if [ "$TARGETARCH" = "amd64" ]; then TARGET=x86_64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm64" ]; then TARGET=aarch64-unknown-linux-musl; \
    elif [ "$TARGETARCH" = "arm" ] && [ "$TARGETVARIANT" = "v7" ]; then TARGET=armv7-unknown-linux-musleabihf; \
    else TARGET=x86_64-unknown-linux-musl; fi; \
    rm -f ./target/$TARGET/release/deps/phone_data*; \
    cargo build --release --target "$TARGET"; \
    cp "target/$TARGET/release/phone_data" "target/release/phone_data"

FROM alpine:3.22

ARG APP=/usr/app

# 安装运行时依赖（ca-certificates 用于 HTTPS，tzdata 用于时区）
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    libgcc \
    && rm -rf /var/cache/apk/*

# 创建非 root 用户
RUN addgroup -g 65532 -S appgroup && adduser -u 65532 -S appuser -G appgroup

ENV TZ=Asia/Shanghai

# 复制编译好的二进制文件和数据文件
COPY --from=builder /phone_data/target/release/phone_data ${APP}/phone_data
COPY --from=builder /phone_data/phone.dat ${APP}/phone.dat

# 设置时区并修改文件所有权
RUN ln -sf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone \
    && chown -R appuser:appgroup ${APP}

WORKDIR ${APP}
USER appuser

CMD ["./phone_data"]
FROM rust:1.84 AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# ビルドしたバイナリだけを軽量なイメージにコピー
COPY --from=builder /app/target/release/notion-ai-webhook /app/server

# Cloud Run ポート設定
ENV PORT=8080
EXPOSE 8080

# 実行コマンド（バイナリ名は Cargo.toml の [package] name に合わせる）
CMD ["/app/server"]

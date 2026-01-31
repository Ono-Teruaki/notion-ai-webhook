FROM rust:1.80 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app
# ビルドしたバイナリだけを軽量なイメージにコピー
COPY --from=builder /app/target/release/notion-diary-ai /app/server

# Cloud Run は環境変数 PORT で指定されたポートを待受ける必要があるため
ENV PORT=8080
EXPOSE 8080

# 実行コマンド（バイナリ名は Cargo.toml の [package] name に合わせる）
CMD ["/app/server"]

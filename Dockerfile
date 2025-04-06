FROM rust:latest AS builder

EXPOSE 8081

WORKDIR snitch-backend
RUN apt update -y && apt install libclang-dev clang llvm-dev -y --no-install-recommends
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runner-slim
WORKDIR snitch-backend
RUN apt update -y && apt install libssl-dev -y --no-install-recommends
COPY --from=BUILDER /snitch-backend/target/release/snitch-backend ./
CMD ["./snitch-backend"]

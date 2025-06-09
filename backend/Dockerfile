FROM rust:latest AS builder

EXPOSE 8081

WORKDIR snitch-backend
RUN apt update -y && apt install libclang-dev clang llvm-dev protobuf-compiler libprotobuf-dev -y --no-install-recommends
COPY . .
RUN cargo build --release --jobs 2

FROM debian:bookworm-slim AS runner-slim
WORKDIR snitch-backend
RUN apt update -y && apt install libssl-dev ca-certificates -y --no-install-recommends && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -r appuser && useradd -r -g appuser appuser
RUN chown -R appuser:appuser /snitch-backend
RUN rm /bin/bash /bin/c_rehash /bin/chsh /bin/dash /bin/rbash /bin/sh

USER appuser

COPY --from=BUILDER /snitch-backend/target/release/snitch-backend ./
CMD ["./snitch-backend"]

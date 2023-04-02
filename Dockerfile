FROM rust:latest AS BUILDER

EXPOSE 8081

WORKDIR snitch-backend
RUN apt update -y && apt install vim libclang-dev -y
COPY . .
RUN cargo build --release --jobs 2

FROM rust:latest AS RUNNER
WORKDIR snitch-backend
COPY --from=BUILDER /snitch-backend/target/debug/snitch-backend ./
CMD ["./snitch-backend"]

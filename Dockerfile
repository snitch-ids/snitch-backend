FROM rust:latest AS BUILDER

WORKDIR snitch-backend
RUN apt update -y && apt install vim libclang-dev -y
COPY . .
RUN cargo build --release

FROM alpine:latest AS RUNNER
WORKDIR snitch-backend
COPY --from=BUILDER /snitch-backend/target/release/snitch-backend ./
CMD ./snitch-backend
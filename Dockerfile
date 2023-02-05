FROM rust:latest

WORKDIR snitch-backend
COPY . .

#RUN cargo install --path .
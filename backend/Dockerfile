FROM rust:1-buster AS builder
WORKDIR /backend
COPY . /backend
RUN cargo build --release


FROM ubuntu:22.04
WORKDIR /backend
COPY --from=builder /backend/target/release/ /backend
EXPOSE 8080
ENTRYPOINT [ "./todo-app-backend" ]

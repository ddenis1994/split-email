FROM rust:1.72-bookworm as builder
LABEL authors="denis_d"
RUN apt-get update && apt-get install -y libclang-dev build-essential
WORKDIR /usr/src/act
COPY . .
RUN cargo build -r

FROM debian:bookworm-slim as runtime
COPY --from=builder /usr/src/act/target/release/act /usr/local/bin/act
EXPOSE 8080
CMD ["act"]
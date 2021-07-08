# build
FROM rust:slim-buster as builder
WORKDIR /code
COPY . .
RUN cargo b --release --no-default-features --features rustls && strip target/release/avabot

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder /code/target/release/avabot .
COPY --from=builder /code/log4rs.yml .
ENTRYPOINT [ "./avabot" ]
CMD []

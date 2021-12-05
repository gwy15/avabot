# build
FROM rust:slim-buster as builder
WORKDIR /code

RUN apt-get update \
    && apt-get install -y clang lld

COPY . .
RUN cargo b --release --no-default-features --features rustls \
    && strip target/release/avabot

# 
FROM debian:buster-slim
WORKDIR /code
COPY log4rs.yml .
COPY --from=builder /code/target/release/avabot .
ENTRYPOINT [ "./avabot" ]
CMD []

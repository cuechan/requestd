FROM debian:sid

RUN apt-get update
RUN apt-get install -y postgresql cargo build-essential libssl-dev

COPY . /ffhl-collector

WORKDIR /ffhl-collector
RUN cargo -V && rustc -V
RUN cargo build
RUN cargo test

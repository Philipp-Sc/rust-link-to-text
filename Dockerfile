#FROM funtoo/stage3-intel64-skylake
FROM rust:latest

RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - \
    && echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google.list
RUN apt-get update && apt-get -y install google-chrome-stable

RUN mkdir /usr/target

WORKDIR /usr/workspace

#ENV RUSTFLAGS="--cfg tokio_unstable"
ENV CARGO_HOME=/usr/cargo_home
ENV CARGO_TARGET_DIR=/usr/target

CMD ["cargo build","cargo run"]



#FROM rust:alpine
#RUN apk add bash alpine musl-dev openssl-dev vim curl alpine-sdk chromium chromium-chromedriver
#RUN rustup target add x86_64-unknown-linux-musl
#RUN rustup component add clippy llvm-tools-preview

#RUN mkdir /usr/target

#ENV CARGO_HOME=/usr/cargo_home
#ENV CARGO_TARGET_DIR=/usr/target

#WORKDIR /usr/workspace

#CMD ["cargo build","cargo run"]

FROM ubuntu:latest
LABEL authors="Gabriel Amihalachioaie"
WORKDIR /cp-config
EXPOSE 3000
RUN apt-get update && \
  apt-get -y install curl unzip &&  \
  mkdir bin && \
  cd bin && \
  curl -LO https://github.com/bitwarden/sdk/releases/download/bws-v0.3.0/bws-x86_64-unknown-linux-gnu-0.3.0.zip && \
  unzip bws-x86_64-unknown-linux-gnu-0.3.0.zip && \
  chmod +x bws && \
  curl -LO https://github.com/microconfig/microconfig/releases/download/v4.9.2/microconfig-linux.zip && \
  unzip microconfig-linux.zip && \
  chmod +x microconfig && \
  export PATH=$PATH:$PWD && \
  cd ../
COPY ./config.yaml ./log4rs.yaml ./cp-config ./
ENTRYPOINT ["./cp-config", "config.yaml"]
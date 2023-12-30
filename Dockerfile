FROM ubuntu:latest
LABEL authors="Gabriel Amihalachioaie"
WORKDIR /cp-config
EXPOSE 3000
COPY ./target/release/config.yaml ./target/release/log4rs.yaml ./target/release/cp-config ./
ENTRYPOINT ["cp-config", "config.yaml"]
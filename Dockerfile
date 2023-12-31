FROM ubuntu:latest
LABEL authors="Gabriel Amihalachioaie"
WORKDIR /cp-config
EXPOSE 3000
COPY ./config.yaml ./log4rs.yaml ./cp-config ./
ENTRYPOINT ["./cp-config", "config.yaml"]
FROM rust:1.75-bookworm AS build
WORKDIR /src
COPY . .
RUN mkdir -p /etc/apt/keyrings && \
  wget -O - https://packages.adoptium.net/artifactory/api/gpg/key/public | tee /etc/apt/keyrings/adoptium.asc && \
  echo "deb [signed-by=/etc/apt/keyrings/adoptium.asc] https://packages.adoptium.net/artifactory/deb $(awk -F= '/^VERSION_CODENAME/{print$2}' /etc/os-release) main" | tee /etc/apt/sources.list.d/adoptium.list && \
  apt-get update && \
  apt-get -y install git temurin-8-jdk ca-certificates && \
  git clone https://github.com/simpleg-eu/bitwarden-sdk.git && \
  cd bitwarden-sdk && \
  cargo build --release && \
  mv ./target/release/bws ./ && \
  cd ../ && \
  git clone https://github.com/simpleg-eu/microconfig.git && \
  cd microconfig && \
  ./gradlew shadowJar && \
  mv ./microconfig-cli/build/libs/microconfig-cli-*-all.jar ./microconfig.jar && \
  ./.github/scripts/native/graalvm-linux.sh && \
  ./.github/scripts/native/native.sh && \
  cd ../ && \
  mkdir bin && \
  mv ./bitwarden-sdk/bws ./bin && \
  mv ./microconfig/microconfig ./bin && \
  cargo build --release

FROM ubuntu:latest AS final
LABEL authors="Gabriel Amihalachioaie"
WORKDIR /cp-config
EXPOSE 3000
ENV PATH="${PATH}:/cp-config/bin"
COPY --from=build /src/bin ./bin
COPY --from=build /src/target/release/cp-config ./
RUN apt-get update && \
    apt-get -y install ca-certificates
ENTRYPOINT ["./cp-config", "/cp-config/config/config.yaml"]
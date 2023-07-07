FROM rust:1-slim-buster AS builder
COPY ./ ./
RUN cargo build --release

FROM oraclelinux:8-slim
RUN rpm --install https://dl.fedoraproject.org/pub/epel/epel-release-latest-8.noarch.rpm \
    && microdnf install -y osslsigncode tpm2-pkcs11 \
    && microdnf clean all
COPY --from=builder ./target/release/signserver /app/signserver
WORKDIR /app/
ENTRYPOINT /app/signserver

FROM ghcr.io/tweedegolf/rust-dev:stable AS builder
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM ghcr.io/tweedegolf/debian:bookworm
# install kubectl
RUN curl -s -L https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key add - \
    && echo "deb http://packages.cloud.google.com/apt cloud-sdk-stretch main" > /etc/apt/sources.list.d/google-cloud-sdk.list \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        google-cloud-sdk \
        kubectl \
    && rm -rf /var/lib/apt/lists/*
# install age
ENV AGE_VERSION=1.1.1
RUN curl -s -L https://github.com/FiloSottile/age/releases/download/v${AGE_VERSION}/age-v${AGE_VERSION}-linux-amd64.tar.gz -o /tmp/age.tar.gz \
    && tar xvf /tmp/age.tar.gz -C /tmp \
    && mv /tmp/age/age /usr/local/bin/age \
    && mv /tmp/age/age-keygen /usr/local/bin/age-keygen \
    && rm -rf /tmp/{age,age.tar.gz}
# install sops
ENV SOPS_VERSION=3.8.1
RUN curl -s -L https://github.com/getsops/sops/releases/download/v${SOPS_VERSION}/sops-v${SOPS_VERSION}.linux -o /usr/local/bin/sops \
    && chmod 0755 /usr/local/bin/sops
# copy executable
COPY --from=builder /app/target/release/kuberwave /app/kuberwave
# run kuberwave
CMD ["/app/kuberwave"]

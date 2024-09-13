FROM ghcr.io/tweedegolf/debian:bookworm

ARG TARGETARCH

# install kubectl
RUN curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" > /etc/apt/sources.list.d/google-cloud-sdk.list \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        kubectl \
    && rm -rf /var/lib/apt/lists/*

# install age
ENV AGE_VERSION=1.2.0
RUN curl -s -L https://github.com/FiloSottile/age/releases/download/v${AGE_VERSION}/age-v${AGE_VERSION}-linux-${TARGETARCH}.tar.gz -o /tmp/age.tar.gz \
    && tar xvf /tmp/age.tar.gz -C /tmp \
    && mv /tmp/age/age /usr/local/bin/age \
    && mv /tmp/age/age-keygen /usr/local/bin/age-keygen \
    && rm -rf /tmp/{age,age.tar.gz}

# install sops
ENV SOPS_VERSION=3.9.0
RUN curl -s -L https://github.com/getsops/sops/releases/download/v${SOPS_VERSION}/sops-v${SOPS_VERSION}.linux.${TARGETARCH} -o /usr/local/bin/sops \
    && chmod 0755 /usr/local/bin/sops

# copy executable
COPY kuberwave.$TARGETARCH /usr/local/bin/kuberwave
RUN chmod 0755 /usr/local/bin/kuberwave

# run kuberwave
CMD ["/usr/local/bin/kuberwave"]

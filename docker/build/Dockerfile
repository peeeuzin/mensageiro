ARG RUST_VERSION=1.74.0

FROM rust:$RUST_VERSION-slim-bullseye as base

WORKDIR /app

ARG \
    # This is needed to fix permissions of files created in
    # the container, so that they are owned by the host user.
    UID=1000 \
    GID=1000

# Install system dependencies.
RUN apt-get update -qq \    
    && apt-get upgrade -y \
    && apt-get install --no-install-recommends -y \
    bash \
    build-essential \
    libssl-dev \
    && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists /var/cache/apt/archives

SHELL ["/bin/bash", "-eo", "pipefail", "-c"]

# We create a non-root user to run the application,
# so that we don't run the application as root.
RUN groupadd -g "${GID}" -r mensageiro \
    && useradd -d '/app' -g mensageiro -l -r -u "${UID}" mensageiro \
    && chown mensageiro:mensageiro -R '/app'

USER mensageiro

COPY --chown=mensageiro:mensageiro Cargo.toml /app/Cargo.toml
COPY --chown=mensageiro:mensageiro src /app/src

# Build the application.
RUN cargo build --release

EXPOSE 6572

FROM base AS release

CMD ["/app/target/release/mensageiro"]
# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
# Taken from https://www.lpalmieri.com/posts/fast-rust-docker-builds/
FROM lukemathwalker/cargo-chef:0.1.51-rust-1.66-bullseye AS chef
ENV WORKING_DIR /usr/src/nstow
WORKDIR "${WORKING_DIR}"

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner "${WORKING_DIR}/recipe.json" recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin nstow

# We do not need the Rust toolchain to run the binary!
FROM debian:12-slim AS runtime

RUN apt-get update \
  && apt-get install --yes \
  jq \
  wget \
  tar \
  && rm -rf /var/lib/apt/lists/*
RUN wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/local/bin/yq \
  && chmod +x /usr/local/bin/yq

COPY --from=builder /usr/src/nstow/target/release/nstow /usr/local/bin/nstow
COPY ./tests/integration-tests /usr/local/bin/integration-tests

ENV USER="stoic"
ENV HOME="/home/${USER}"
RUN useradd \
  --create-home \
  --home-dir "${HOME}" \
  --shell /bin/bash \
  --uid 1000 \
  --password '' \
  "${USER}"
USER "${USER}"

ENV XDG_CONFIG_HOME="${HOME}/.config"

WORKDIR "${HOME}"
COPY examples ./examples

ENTRYPOINT ["integration-tests"]


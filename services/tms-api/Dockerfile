# Stage 1: Build the client
# REDIRECT_URI is compiled into the bundle (the login link sends it to /login, which checks
# it against the allowed redirects), so each image is built for one environment.  Without
# it every login would be rejected, so the build stops instead.
FROM node:24-slim AS node-build

ARG REDIRECT_URI
# keep in step with the pnpm version in .github/workflows/client-ci.yml
ARG PNPM_VERSION=12.6.0
RUN test -n "${REDIRECT_URI}" || \
    (echo "The REDIRECT_URI build arg is required, e.g. --build-arg REDIRECT_URI=https://<portal host>/" >&2 && exit 1)
RUN npm install -g pnpm@${PNPM_VERSION}

WORKDIR /client

COPY ./client/package.json \
     ./client/pnpm-lock.yaml \
     ./client/pnpm-workspace.yaml \
     ./

RUN pnpm install --frozen-lockfile

COPY ./client/ ./

RUN pnpm run build


# Stage 2: Build the API
FROM rust@sha256:80302520b7199f0504975bca59a914015e9fee088f759875dbbc238ca9509ee1 AS rust-build

ARG SERVICE_NAME
WORKDIR /usr/local/${SERVICE_NAME}

COPY . .
RUN cargo build --release -p ${SERVICE_NAME} --locked


# Stage 3: Runtime
FROM debian:trixie-slim

ARG SERVICE_NAME

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 && \
    update-ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*

COPY --from=rust-build \
    /usr/local/${SERVICE_NAME}/target/release/${SERVICE_NAME} \
    /usr/local/bin/${SERVICE_NAME}

# the server serves the UI from dist/ relative to its working directory (main.rs)
WORKDIR /usr/local/client

COPY --from=node-build /client/dist ./dist

ENV RUST_LOG=debug,shared=debug
ENV SERVICE_NAME=${SERVICE_NAME}

CMD ["sh", "-c", "exec /usr/local/bin/$SERVICE_NAME"]

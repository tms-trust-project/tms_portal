# Stage 1: Build the client
FROM node:24 AS node-build

RUN npm install -g pnpm

WORKDIR /client

COPY ./client/package.json \
     ./client/pnpm-lock.yaml \
     ./client/pnpm-workspace.yaml \
     ./

RUN pnpm install --frozen-lockfile

COPY ./client/ ./

ARG REDIRECT_URI
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

WORKDIR /usr/local/client

COPY --from=node-build /client/dist ./dist

ENV RUST_LOG=debug,shared=debug
ENV SERVICE_NAME=${SERVICE_NAME}

CMD ["sh", "-c", "exec /usr/local/bin/$SERVICE_NAME"]
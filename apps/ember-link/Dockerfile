#### BASE
FROM node:21.7.3 AS base
WORKDIR /app
ENV MOON_TOOLCHAIN_FORCE_GLOBALS=true

RUN apt-get update && apt-get install -y protobuf-compiler libprotobuf-dev

RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=1.84.0
ENV PATH="/root/.cargo/bin:${PATH}"

# Install moon binary
RUN npm install -g @moonrepo/cli
### BASE

#### WORKSPACE
FROM base AS workspace
WORKDIR /app

# Copy entire repository and scaffold
COPY . .
RUN moon docker scaffold ember-link

### CHEF
FROM base AS chef
RUN cargo install cargo-chef
### CHEF

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ember-link

#### BUILD
FROM planner AS build
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

# Copy workspace skeleton
COPY --from=workspace /app/.moon/docker/workspace .
# Remove unneeded cargo binaries
RUN sed -i '/cargo-llvm-cov/d' /app/.moon/toolchain.yml
# Install toolchain and dependencies
RUN moon docker setup

# Copy source files
COPY --from=workspace /app/.moon/docker/sources .

# Run something
RUN moon ember-link:build-release

RUN cp ./target/release/ember-link .

# Prune workspace
RUN moon docker prune
### BUILD

FROM gcr.io/distroless/cc

COPY --from=build /app/ember-link .

CMD ["./ember-link"]
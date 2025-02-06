# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.83.0
ARG APP_NAME=coc-dash

################################################################################
# Build Stage

FROM rust:${RUST_VERSION}-bookworm AS build
ARG APP_NAME
WORKDIR /app

# Install dependencies required for compilation
RUN apt update && apt install -y clang lld git libssl-dev

# Copy source files
COPY . .

# Cache dependencies to speed up builds
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

################################################################################
# Runtime Stage

FROM debian:bookworm AS final

RUN apt update && apt install -y ca-certificates

# Create a non-privileged user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Set working directory
WORKDIR /app

# Create persistent storage for battles.json
VOLUME ["/app"]

COPY .env /app/.env

RUN mkdir -p /app/data
COPY data /app/data

RUN mkdir -p /app/templates
COPY templates /app/templates

RUN mkdir -p /app/static
COPY static /app/static

# Copy the built binary from the build stage
COPY --from=build /bin/server /bin/

# Expose the port the app runs on
EXPOSE 8080

# Run the application
CMD ["/bin/server"]


FROM rust:1.74.0-slim-bullseye AS build
ARG APP_NAME=waft
WORKDIR /app
VOLUME /data

RUN apt update && apt install -y build-essential libssl-dev pkg-config libpq-dev

# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=templates,target=templates \
    --mount=type=bind,source=example.toml,target=config.toml \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release

cp ./target/release/$APP_NAME /bin/$APP_NAME
cp -r ./assets /bin/
cp ./config.toml /bin/
EOF

FROM debian:bullseye-slim AS final

# RUN mkdir /data

# Copy the executable from the "build" stage.
COPY --from=build /bin/waft /bin/
COPY --from=build /bin/assets /bin/assets
COPY --from=build /bin/config.toml /data/config.toml

# What the container should run when it is started.
# CMD ["/bin/waft", "--assets", "/bin/assets", "--config", "/data/config.toml"]


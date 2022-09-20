FROM lukemathwalker/cargo-chef:latest-rust-1.63 AS chef

RUN apt-get update
RUN apt-get install -y build-essential libclang-dev

WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin experimental-cms

# We do not need the Rust toolchain to run the binary!
FROM ubuntu AS runtime
WORKDIR app
RUN apt-get update &&  apt-get install -y curl
COPY --from=builder /app/target/release/experimental-cms /usr/local/bin
COPY --from=builder /app/js /app/js
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/experimental-cms"]
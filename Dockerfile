# Build using ShareSphere builder
FROM ghcr.io/winteler/sharesphere-builder:main AS builder

WORKDIR /sharesphere
COPY . .

RUN npm install & npm run build

RUN cargo leptos build --release --precompress

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

# Install any needed runtime libs
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates rsync && apt-get clean

# Copy binary from builder
COPY --from=builder /sharesphere/target/release/server /usr/local/bin/sharesphere
COPY --from=builder /sharesphere/target/site /usr/local/bin/site

ENV LEPTOS_OUTPUT_NAME="sharesphere"
ENV LEPTOS_SITE_ROOT="/usr/local/bin/site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"

EXPOSE 3000
ENTRYPOINT ["sharesphere"]
# Sharesphere

## Setting up Sharesphere

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
3. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
4. `cargo install cargo-leptos` - install `cargo-laptos` binary
5. `npm install` - install `TailwindCSS` and `DaisyUI`
6. Add a `.env` file in the repo's root folder with your Postgres connection, e.g. `DATABASE_URL=postgres://<user>:<password>@<postgres_url>/<schema_name>`
7. `cargo install sqlx-cli --no-default-features --features rustls,postgres` - Install sqlx-cli
8. `sqlx migrate run` - perform migrations on the DB
9. Set the following environment variables:
   * OIDC_ISSUER_ADDR - url of the keycloak instance
   * AUTH_CLIENT_ID - ID of the client in Keycloak
   * AUTH_CLIENT_SECRET - Secret of the client in Keycloak
   * DATABASE_URL - Database url
   * SESSION_KEY - Key to persist session data
   * SESSION_DB_KEY - DB key to persist session data
   * TEST_DATABASE_URL - Test DB url, used in integration tests

## Running Sharesphere

```bash
cargo leptos watch
```

## Compiling for Release
```bash
cargo leptos build --release
```

Will generate your server binary in target/server/release and your site package in target/site

## Testing

### Unit & integration tests
```bash
cargo test -F ssr
```

### End-to-end tests
Run `npm install` in the end2end subdirectory before testing
```bash
cargo leptos end-to-end
```

```bash
cargo leptos end-to-end --release
```

Cargo-leptos uses Playwright as the end-to-end test tool.  
Tests are located in end2end/tests directory.

## Executing a Server on a Remote Machine Without the Toolchain
After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/release/server`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:
```text
start-axum
site/
```
Set the following environment variables (updating for your project as needed):
```text
LEPTOS_OUTPUT_NAME="start-axum"
LEPTOS_SITE_ROOT="site"
LEPTOS_SITE_PKG_DIR="pkg"
LEPTOS_SITE_ADDR="127.0.0.1:3000"
LEPTOS_RELOAD_PORT="3001"
```
Finally, run the server binary.

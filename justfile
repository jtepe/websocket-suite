# run source checks
check:
    cargo check

# lint code
lint:
    cargo clippy

# format code
fmt:
    cargo fmt

# runs all tests
test:
    cargo test

# runs specific test
ptest TEST:
    cargo test {{TEST}}

# build all member packages
build:
    cargo build

# execute synchronous client with specific wss URL
run-sync-client URL:
    cargo run -p synchronous-client {{URL}}

# generate sign-signed certificate in cert.pem
genkey:
    cargo run -p genkey

# clean artifacts
clean:
    cargo clean

# start the podman compose services (server + client)
# By default this does NOT rebuild images.
# Pass BUILD=true to rebuild (e.g. `just compose-up true`).
# Stops when one of the containers exits (useful for this short-lived test setup)
compose-up BUILD="false":
    if [ "{{BUILD}}" = "true" ] || [ "{{BUILD}}" = "1" ]; then \
      podman compose up --build --abort-on-container-exit; \
    else \
      podman compose up --abort-on-container-exit; \
    fi

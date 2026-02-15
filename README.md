# WebSocket TLS test workspace (server + client)

This workspace contains:

- `genkey`: generates a self-signed TLS certificate for `localhost` (`cert.pem`, `key.pem`)
- `synchronous_server`: single-connection WSS (TLS) websocket server
- `synchronous_client`: single-run WSS websocket client (sends a few messages then exits)

## Generate certificates (run once)

```bash
just genkey
```

This writes `./cert.pem` and `./key.pem` (mounted into the containers).

## Run via Podman Compose

```bash
# run without rebuilding images
just compose-up

# rebuild images then run
just compose-up true
```

Both containers run with `network_mode: host` so that everything is reachable via the host's `localhost` (useful when a proxy runs on the host).

(Internally, `just compose-up` uses `podman compose --podman-run-args="--rm" up --abort-on-container-exit` so the containers are automatically removed once they exit.)

### Configuration

Environment variables (can be passed via your shell or a `.env` file):

- `SERVER_PORT` (default `3043`) – port the server listens on (host network)
- `WS_BIND` (default `127.0.0.1`) – server bind address; set to `0.0.0.0` if you want to expose beyond localhost
- `CLIENT_HOST` (default `localhost`) – host the client connects to
- `CLIENT_PORT` (default `3043`) – port the client connects to

For proxy testing you typically point the client at your proxy (e.g. `CLIENT_PORT=PROXY_PORT`) and run the server on a different `SERVER_PORT`.

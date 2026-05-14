# **Docker**

The Ember Link server is published as a Docker image on Docker Hub at `emberlinkio/ember-link`. The image runs a single Tokio-based Rust binary that listens on the configured host and port, reads its configuration from environment variables, and accepts WebSocket connections from the client SDK.

## **Pulling the Image**

```sh copyButton
docker pull emberlinkio/ember-link:latest
```

The `latest` tag tracks the most recent published release. Tag a specific version (for example `emberlinkio/ember-link:0.1.0`) for production deployments where you want pinned, reproducible builds.

## **Running for Prototyping**

The following command starts the server on port `9000`, bound to every interface, with authentication disabled. This is the configuration used by the project's example applications.

```sh copyButton
docker run -d -p 9000:9000 \
	--env PORT=9000 \
	--env HOST=0.0.0.0 \
	--env ALLOW_UNAUTHORIZED=true \
	emberlinkio/ember-link:latest
```

`ALLOW_UNAUTHORIZED=true` causes the server to accept WebSocket connections without a JWT. This setting is acceptable for local development but must not be used in production.

## **Running for Production**

A production deployment needs a JWT signer key and authentication enabled. The signer key is an RSA public key in PEM format. The corresponding private key lives on your backend and is used to sign the tokens your `authEndpoint` returns to the SDK.

```sh copyButton
docker run -d -p 9000:9000 \
	--env PORT=9000 \
	--env HOST=0.0.0.0 \
	--env JWT_SIGNER_KEY="$(cat /path/to/jwt-public-key.pem)" \
	emberlinkio/ember-link:latest
```

Behind a reverse proxy or load balancer, terminate TLS at the proxy and forward `Upgrade: websocket` headers to the container. Ember Link's WebSocket endpoint is `/ws`. Health checks should target `/`.

## **Persistent Storage**

The Docker image persists channel storage to its working directory by default. Mount a volume to keep storage across container restarts.

```sh copyButton
docker run -d -p 9000:9000 \
	-v ember-link-data:/data \
	--env STORAGE_ENDPOINT=/data \
	--env JWT_SIGNER_KEY="$(cat /path/to/jwt-public-key.pem)" \
	emberlinkio/ember-link:latest
```

`STORAGE_ENDPOINT` accepts either a local filesystem path or the URL of an external storage service. The format of the URL is documented in [Server config](/self-hosting/config).

## **Environment Variables**

Every supported environment variable, default value, and description is listed in [Server config](/self-hosting/config). The most common values are summarised below for reference.

| **Variable**          | **Default**     | **Description**                                                                                                                          |
| --------------------- | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `HOST`                | `127.0.0.1`     | Interface to bind on. Set to `0.0.0.0` inside Docker to accept connections from outside the container.                                    |
| `PORT`                | `8787`          | Port to listen on. The project examples use `9000`.                                                                                       |
| `ALLOW_UNAUTHORIZED`  | `false`         | When `true`, the server accepts WebSocket connections without a JWT. For prototyping only.                                                |
| `JWT_SIGNER_KEY`      | none            | RSA public key (PEM format) the server uses to verify incoming JWTs.                                                                      |
| `STORAGE_ENDPOINT`    | none            | Where to persist channel storage. Accepts a filesystem path or an external service URL.                                                   |

## **docker-compose**

A minimal `docker-compose.yml` for production:

```yaml copyButton
services:
  emberlink:
    image: emberlinkio/ember-link:latest
    restart: unless-stopped
    ports:
      - '9000:9000'
    environment:
      PORT: '9000'
      HOST: '0.0.0.0'
      JWT_SIGNER_KEY: '${EMBERLINK_JWT_SIGNER_KEY}'
      STORAGE_ENDPOINT: '/data'
    volumes:
      - ember-link-data:/data

volumes:
  ember-link-data:
```

## **Related**

- [Server config](/self-hosting/config) for the full list of environment variables.
- [Cloudflare Workers](/self-hosting/cloudflare-workers) for the edge deployment target.
- [Concepts → Client](/concepts/client) for the client-side authentication flow that pairs with `JWT_SIGNER_KEY`.

# **Server Config**

The Ember Link server reads every configuration value from the process environment. The Docker target reads these as standard environment variables. The Cloudflare Workers target reads them from the `[vars]` table in `wrangler.toml`. The complete list is documented below, along with which Cargo feature flag is required for each.

## **Tokio (Docker) Configuration**

The Tokio-based binary inside the Docker image accepts two additional values that control the listening socket. These are not used by the Cloudflare Workers target.

| **Variable** | **Default**     | **Description**                                                                                                                                                                                |
| ------------ | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `HOST`       | `127.0.0.1`     | Interface to bind on. Set this to `0.0.0.0` inside a Docker container to accept connections from outside the container.                                                                         |
| `PORT`       | `8787`          | Port to listen on. The project examples expose the container on port `9000` and set `PORT=9000` to match, so the WebSocket URL the client uses (`http://localhost:9000`) matches the bind port. |

## **Shared Configuration**

These values are read by both the Tokio (Docker) and Cloudflare Workers targets.

### **Authentication**

| **Variable**           | **Default** | **Description**                                                                                                                                                                                |
| ---------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ALLOW_UNAUTHORIZED`   | `false`     | When `true`, the server accepts WebSocket connections without a JWT and skips signature verification. Intended for local development only. Production deployments must leave this value unset or `false`. |
| `JWT_SIGNER_KEY`       | none        | RSA public key in PEM format. The server verifies the signature of every incoming JWT against this key. Required for production deployments. The corresponding private key lives on your own backend and is used to sign the tokens that your `authEndpoint` returns. |

### **Multi-tenant Operation (Cargo feature `multi-tenant`)**

| **Variable**                | **Default** | **Description**                                                                                                                                              |
| --------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `JWT_SIGNER_KEY_ENDPOINT`   | none        | URL the server calls with a tenant identifier (`tenant_id`) to retrieve the signer key for that tenant. Enables per-tenant signing keys in multi-tenant deployments. |

### **Webhooks (Cargo feature `webhook`)**

| **Variable**         | **Default** | **Description**                                                                                                                                       |
| -------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `WEBHOOK_URL`        | none        | URL the server posts webhook events to. Required when the `webhook` feature is enabled.                                                                |
| `WEBHOOK_SECRET_KEY` | none        | Secret used to sign webhook payloads so the receiver can verify them. Required when the `webhook` feature is enabled.                                  |

### **Storage**

| **Variable**       | **Default** | **Description**                                                                                                                                                          |
| ------------------ | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `STORAGE_ENDPOINT` | none        | Where to persist channel storage. On the Docker target this accepts a local filesystem path or the URL of an external storage service. On the Cloudflare Workers target storage lives inside each channel's Durable Object and this value is not required. |

## **JWT Token Shape**

The JWT returned by your `authEndpoint` and verified by the server with `JWT_SIGNER_KEY` must be signed with `RS256` and include the following claims at minimum.

| **Claim** | **Type** | **Description**                                                                                                                                              |
| --------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `uid`     | `string` | Unique identifier for the connecting user. The server uses this as the `clientId` published to other peers in the channel.                                   |
| `iat`     | `number` | Standard JWT issued-at claim, in seconds since the epoch.                                                                                                    |
| `exp`     | `number` | Standard JWT expiration claim, in seconds since the epoch. The SDK caches valid tokens until 30 seconds before this value.                                   |

The server may reject tokens that are missing required claims, that fail signature verification, or that are expired. The application-level WebSocket close codes returned in these cases are documented in [@ember-link/protocol](/packages/protocol).

## **Cargo Feature Flags**

The features that determine which environment variables are required are compile-time options on the server crate, not runtime values. The published Docker image is built with all features enabled. Custom builds can disable features they do not use to reduce the binary size.

| **Feature**     | **Required For**                                                                                                                                |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `webhook`       | `WEBHOOK_URL` and `WEBHOOK_SECRET_KEY`. Enables the outgoing webhook event system.                                                              |
| `multi-tenant`  | `JWT_SIGNER_KEY_ENDPOINT`. Enables per-tenant signing keys, looked up at connection time from the configured endpoint.                          |

## **Related**

- [Docker](/self-hosting/docker) for the traditional deployment target.
- [Cloudflare Workers](/self-hosting/cloudflare-workers) for the edge deployment target.
- [Concepts → Client](/concepts/client) for the client-side authentication flow.

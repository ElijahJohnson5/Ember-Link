# **Self-hosting**

Ember Link is designed to be self-hosted. The server is published as a Docker image for traditional infrastructure, and as a Rust crate compiled to WebAssembly for Cloudflare Workers and Durable Objects. This section walks through both deployment paths and documents the full set of configuration values the server accepts.

## **Deployment Options**

Two server targets are supported today. Both targets implement the same wire protocol and accept the same client SDK without changes.

### **[Docker](/self-hosting/docker)**

A single Docker image (`emberlinkio/ember-link:latest`) that runs on any platform that can host containers. The image listens on a port you choose, persists channel storage to the local filesystem by default, and reads its configuration from environment variables. This is the recommended path for traditional VM and Kubernetes hosting.

### **[Cloudflare Workers](/self-hosting/cloudflare-workers)**

A Cloudflare Workers deployment that runs Ember Link at the edge. Each channel is backed by a Durable Object, which gives the channel a single canonical location and a built-in persistent store. The Workers target is the recommended path when your application is already on Cloudflare's platform.

## **Configuration**

Both deployment targets share the same configuration values, although the mechanism for setting them differs (environment variables for the Docker target, `[vars]` and bindings in `wrangler.toml` for the Workers target). See [Server config](/self-hosting/config) for the full reference.

## **Authentication**

In production the server requires every client to present a signed JWT. The token is requested by the SDK from your own backend by way of the `authEndpoint` client option, and is verified by the server against the `JWT_SIGNER_KEY` it was started with. See [Concepts → Client](/concepts/client) for the client-side authentication flow, and [Server config](/self-hosting/config) for the server-side configuration.

For prototyping, set `ALLOW_UNAUTHORIZED=true`. The server then accepts unauthenticated connections. Do not set this flag in production.

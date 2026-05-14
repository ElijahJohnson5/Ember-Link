# **Cloudflare Workers**

The Cloudflare Workers target compiles the Ember Link server to WebAssembly and runs it on Cloudflare's edge network. Each channel is backed by a Durable Object, which gives the channel a single canonical location for coordination and a built-in persistent store. The Workers target is the recommended deployment path when your application is already on Cloudflare's platform.

The source for the Workers entry point lives at `apps/cloudflare-ember-link` in the project repository. Most production deployments fork or vendor this entry point and adjust `wrangler.toml` to match their account and routes.

## **Prerequisites**

Before deploying, install the [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/), authenticate it with `wrangler login`, and confirm that your account has Durable Objects and Queues enabled. The webhook event queue requires Cloudflare Queues, which is on a paid plan.

The Workers entry point is a Rust crate compiled with [`worker-build`](https://github.com/cloudflare/workers-rs). The build also requires a working Rust toolchain with the `wasm32-unknown-unknown` target.

## **wrangler.toml**

A minimal `wrangler.toml` for the Workers entry point looks like the following.

```toml copyButton
name = "emberlink-on-workers"
main = "build/worker/shim.mjs"
compatibility_date = "2023-03-22"

[vars]
ALLOW_UNAUTHORIZED = "true"
WEBHOOK_URL = "https://your-webhook-receiver.example.com"

[build]
command = "cargo install worker-build && RUSTFLAGS='--cfg getrandom_backend=\"wasm_js\"' worker-build --release"

[durable_objects]
bindings = [
  { name = "CHANNEL", class_name = "CloudflareChannel" }
]

[[migrations]]
tag = "v1"
new_classes = ["CloudflareChannel"]

[[queues.consumers]]
queue = "webhook-events"
max_batch_size = 100
max_batch_timeout = 5

[[queues.producers]]
queue = "webhook-events"
binding = "WEBHOOK_EVENTS"
```

The values above match the reference `wrangler.toml` shipped in `apps/cloudflare-ember-link/wrangler.toml`. Replace `ALLOW_UNAUTHORIZED = "true"` and the placeholder `WEBHOOK_URL` before deploying to production.

## **Durable Objects**

The `CHANNEL` Durable Object binding points at the `CloudflareChannel` class compiled into the worker. Every Ember Link channel maps to a single Durable Object instance, so all clients connected to the same channel name route to the same Durable Object. The Durable Object holds the channel's state and its CRDT storage.

The `[[migrations]]` block above registers `CloudflareChannel` as a new class on tag `v1`. Any subsequent renames or removals require a new migration block with a unique tag.

## **Queues**

The Webhook system uses Cloudflare Queues to buffer outgoing event deliveries. The `WEBHOOK_EVENTS` producer binding and matching consumer in `wrangler.toml` create and consume the `webhook-events` queue, with batches of up to 100 events flushed every 5 seconds.

If your deployment does not use webhooks, the queue bindings can be removed from `wrangler.toml` and the matching feature flags adjusted in `Cargo.toml`.

## **Deploying**

Run a development server locally with Miniflare:

```sh copyButton
yarn workspace emberlink-on-workers dev
```

Or deploy to Cloudflare:

```sh copyButton
yarn workspace emberlink-on-workers deploy
```

The deploy command builds the worker, uploads the WASM binary, and applies any pending Durable Object migrations.

## **Configuration**

The Workers target reads its configuration from the `[vars]` table in `wrangler.toml` (which Cloudflare exposes as environment variables to the worker) and from bindings (Durable Objects, Queues). The supported environment-variable values are the same as those documented in [Server config](/self-hosting/config), with the exception of `HOST` and `PORT`, which do not apply to the edge runtime.

## **Persistent Storage**

Channel storage on the Workers target lives inside the corresponding Durable Object's storage. The data is replicated by Cloudflare's infrastructure and is durable across worker restarts and Durable Object hibernation. No external `STORAGE_ENDPOINT` is required.

## **Related**

- [Server config](/self-hosting/config) for the full list of environment variables.
- [Docker](/self-hosting/docker) for the traditional deployment target.
- [Concepts → Client](/concepts/client) for the client-side authentication flow.

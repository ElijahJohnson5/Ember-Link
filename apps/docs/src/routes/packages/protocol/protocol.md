# **@ember-link/protocol**

`@ember-link/protocol` defines the wire protocol that the client SDK and the Rust server use to talk to each other. The package contains code generated from the project's BARE schema, a small set of TypeScript bindings generated from the server's Rust types, and an enum of the application-level WebSocket close codes that the server can emit.

`@ember-link/protocol` is consumed internally by `@ember-link/core`. Most applications do not need to import it directly. The package is published separately so that authors of custom server implementations or non-JavaScript clients can match the on-wire schema exactly.

## **Installation**

```sh copyButton
yarn add @ember-link/protocol
```

## **WebSocketCloseCode**

The server emits one of the following close codes when it terminates a WebSocket connection for an application-level reason. The codes are in the IANA private-use range (`4000`–`4999`) and do not conflict with standard WebSocket close codes.

```typescript copyButton
enum WebSocketCloseCode {
	TokenNotFound        = 4000,
	InvalidToken         = 4001,
	InvalidSignerKey     = 4002,
	ChannelCreationFailed = 4003,
	MissingTenantId      = 4004
}
```

| **Code** | **Name**                | **Description**                                                                                                |
| -------- | ----------------------- | -------------------------------------------------------------------------------------------------------------- |
| `4000`   | `TokenNotFound`         | The connection request did not include a token, and the server is not running with `ALLOW_UNAUTHORIZED=true`. |
| `4001`   | `InvalidToken`          | The supplied token was not a valid JWT, or its signature did not verify against the server's signer key.       |
| `4002`   | `InvalidSignerKey`      | The server's configured signer public key could not be parsed.                                                 |
| `4003`   | `ChannelCreationFailed` | The server failed to create or look up the requested channel.                                                  |
| `4004`   | `MissingTenantId`       | The server is running in multi-tenant mode but the request did not include a tenant identifier.                |

## **Generated Types**

The remainder of the package re-exports the types generated from the BARE schema (the message envelopes sent between client and server) and the TypeScript bindings generated from the server's Rust types (such as the JWT access token shape).

Direct use of these types is rarely necessary from application code. If you do need them, import them from `@ember-link/protocol` at the top level. Refer to the package's TypeScript definitions and the [project README](https://github.com/ElijahJohnson5/Ember-Link) for the current set of generated types.

## **Related**

- [@ember-link/core](/packages/core) for the high-level client APIs that wrap the protocol.
- [Self-hosting → Server config](/self-hosting/config) for the server-side configuration referenced by the close codes above.

# **Client** API Reference

The `Client` object is the main entry point for interacting with Ember Link's real-time collaboration platform. It is able to create **Channels** that connect to the specified URL, handles authentication and multi tenant support.

---

## Creating a Client

To create a new instance of `Client`, you must pass a configuration object of type `CreateClientOptions`.

### CreateClientOptions

```typescript
interface CreateClientOptions {
	baseUrl: string;
	authEndpoint?: AuthEndpoint;
	multiTenant?: {
		tenantId: string;
	};
	jwtSignerPublicKey?: string;
}
```

| Name               | Type         | Required | Description                                              |
| ------------------ | ------------ | -------- | -------------------------------------------------------- |
| baseUrl            | string       | ✅       | The base URL of your Ember Link backend.                 |
| authEndpoint       | AuthEndpoint | ❌       | Configuration for authentication.                        |
| └ URL              | string       | ❌       | A URL for the authentication endpoint.                   |
| └ function         | function     | ❌       | A function that returns a signed JWT for authentication. |
| multiTenant        | Object       | ❌       | Configuration for multi-tenant setup.                    |
| └ tenantId         | string       | ❌       | The tenant ID used in a multi-tenant setup.              |
| jwtSignerPublicKey | string       | ❌       | Public key used to verify JWTs from your auth provider.  |

### Methods

```typescript
type JoinChannel<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
> = <S extends IStorageProvider>(
	channelName: string,
	options?: ChannelConfig<S, P>['options']
) => { channel: Channel<P, C>; leave: () => void };
```

channelName: The name of the channel to join.

options: An optional configuration for joining the channel.

Returns a Channel object and a leave function to leave the channel

Type Arguments: In most cases you shouldn't have to manually pass the type arguments. See [global types](/global-types) for more information.

### Example Usage

```typescript
import { Client } from '@ember-link/core';

const client = new Client({
	baseUrl: 'https://localhost:9000',
	multiTenant: { tenantId: 'team-123' }
});

const { channel, leave } = await client.joinChannel('editor-room', {
	initialPresence: { username: 'Julia' }
});

// Call leave when you are done with the channel
leave();
```

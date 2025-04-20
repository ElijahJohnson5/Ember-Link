# @ember-link/core

## What is @ember-link/core?

> **@ember-link/core** is the foundational package for all Ember Link clients. It provides the low-level primitives and protocols that power real-time communication across all supported frameworks (React, Svelte, etc).

### Key Responsibilities

- WebSocket Management
  Establishes and maintains a persistent connection to the Ember Link server with automatic reconnection and backoff strategies.

- Channel Layer
  A lightweight publish/subscribe interface that lets you join, leave, and communicate over named channels.

- Authentication
  Token-based client authentication for secure access to collaborative resources.

### When to Use @ember-link/core

- Building a custom framework integration

- Extending or debugging low-level behavior

- Using plain Javascript

## Installation

```bash copyButton
yarn install @ember-link/core
```

## Basic Usage

```ts copyButton
import { createClient } from '@ember-link/core';

const client = createClient({
	baseUrl: 'http://localhost:9000'
});

// 3. Join a channel (e.g., a collaborative document or room)
const { channel, leave } = client.joinChannel('test');

// 4. Listen to other user events
channel.events.subscribe('others', (others) => {
	console.log('Current users in channel: ', others);
});

// 5. Update your own presence
channel.updatePresence({ status: 'online' });

// 6. Listen to your own presence
channel.events.subscribe('presence', (presence) => {
	console.log('My Presence was updated:', presence);
});

// 7. Send any message you want
channel.sendCustomMessage({ data: 'can be any JSON serializable data' });

// 8. Listen to custom messages
channel.events.subscribe('customMessage', (message) => {
	console.log('Recieved message from peers: ', message);
});
```

### <a id="typescript" href="#typescript">Typescript</a>

Defining custom presence types and custom message types

```ts copyButton
declare global {
	interface EmberLink {
		Presence: {
			status: 'online' | 'offline';
		};
		Custom: {
			data: string;
		};
	}
}
```

## API

### Client

To create a new instance of `Client`, you must pass a configuration object of type `CreateClientOptions`.

#### CreateClientOptions

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

#### Methods

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

Type Arguments: In most cases you shouldn't have to manually pass the type arguments and you should be able to define them like in the [typescript example above](#typescript)

### Connecting to a Channel

To create a new instance of a `Channel`, you need to call the `joinChannel` function on a Client

### ChannelConfig

```typescript
interface ChannelConfig<
	S extends IStorageProvider,
	P extends Record<string, unknown> = DefaultPresence
> {
	channelName: string;
	baseUrl: string;
	authenticate: () => Promise<AuthValue>;
	createWebSocket: (authValue: AuthValue) => WebSocket;
	options?: {
		initialPresence?: P;
		storageProvider?: S;
		autoConnect?: boolean;
	};
}
```

| **Name**          | **Type**         | **Required** | **Description**                                                    |
| ----------------- | ---------------- | ------------ | ------------------------------------------------------------------ |
| options           | Object           | ❌           | Optional configuration for the channel.                            |
| └ initialPresence | Type of Presence | ❌           | Optional initial presence state for the channel.                   |
| └ storageProvider | S                | ❌           | Optional storage provider for syncing data.                        |
| └ autoConnect     | boolean          | ❌           | Whether to automatically connect to the channel (default: `true`). |

#### Interface

```typescript
type ChannelEvents<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
> = {
	presence: (self: P) => void;
	status: (status: Status) => void;
	others: (others: User<P>[]) => void;
	destroy: () => void;
	customMessage: (message: Extract<ServerMessage<P, C>, { type: 'custom' }>['data']) => void;
};

type YjsProviderEvents = {
	syncMessage: (message: StorageSyncMessage) => void;
	updateMessage: (message: StorageUpdateMessage) => void;
};

type Channel<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
> = {
	updatePresence: (state: P) => void;
	sendCustomMessage: (data: C) => void;
	hasStorage: () => boolean;
	getStorage: () => IStorage;
	getStatus: () => Status;
	getOthers: () => User<P>[];
	getName: () => string;
	getPresence: () => P | null;
	destroy: () => void;
	updateYDoc: (data: StorageUpdateMessage) => void;
	syncYDoc: (data: StorageSyncMessage) => void;
	events: Observable<ChannelEvents<P, C>> & {
		others: Observable<OtherEvents<P>>;
		yjsProvider: Observable<YjsProviderEvents>;
	};
};
```

| **Name**            | **Type**                               | **Description**                                                                 |
| ------------------- | -------------------------------------- | ------------------------------------------------------------------------------- |
| `updatePresence`    | `(state: P) => void`                   | Updates the local users presence in the channel.                                |
| `sendCustomMessage` | `(data: C) => void`                    | Sends a message to all other users in the channel.                              |
| `hasStorage`        | `() => boolean`                        | Returns true if the channel has a storage provider, false otherwise.            |
| `getStorage`        | `() => IStorage`                       | Retrieves the channel's storage provider.                                       |
| `getStatus`         | `() => Status`                         | Returns the current connection status of the channel.                           |
| `getOthers`         | `() => User<P>[]`                      | Retrieves the list of other users in the channel.                               |
| `getName`           | `() => string`                         | Returns the name of the channel.                                                |
| `getPresence`       | `() => P`                              | Returns the presence state of the channel or null if unavailable.               |
| `destroy`           | `() => void`                           | Destroys the channel, cleaning up resources and connections.                    |
| `updateYDoc`        | `(data: StorageUpdateMessage) => void` | Sends an update to the Yjs document in the channel.                             |
| `syncYDoc`          | `(data: StorageSyncMessage) => void`   | Synchronizes the Yjs document in the channel.                                   |
| `events`            | `Observable<ChannelEvents<P, C>>`      | Event emitter for channel-specific events. Includes `others` and `yjsProvider`. |

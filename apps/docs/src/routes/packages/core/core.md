# **@ember-link/core**

`@ember-link/core` is the foundational package of the Ember Link SDK. The package implements the WebSocket connection, the authentication flow, the channel lifecycle, the presence and storage syncing protocol, and the event subscription API. The React and Svelte integrations build on top of `@ember-link/core` and re-export every type and runtime export it provides, so most applications using a framework integration do not need to depend on `@ember-link/core` directly.

## **When to Use This Package**

- Plain JavaScript or TypeScript applications without a UI framework.
- Custom framework integrations (for example, Vue, Solid, Qwik).
- Test harnesses and tooling that need a real client without UI bindings.

## **Installation**

```sh copyButton
yarn add @ember-link/core
```

## **Basic Usage**

```typescript copyButton
import { createClient } from '@ember-link/core';

const client = createClient({
	baseUrl: 'http://localhost:9000'
});

const { channel, leave } = client.joinChannel('test', {
	presenceThrottle: 33
});

channel.events.subscribe('others', (others) => {
	console.log('Current users in channel:', others);
});

channel.events.subscribe('presence', (presence) => {
	console.log('My presence was updated:', presence);
});

channel.updatePresence({ status: 'online' });

channel.sendCustomMessage({ data: 'can be any JSON-serializable data' });

channel.events.subscribe('customMessage', (message) => {
	console.log('Received message from peers:', message);
});

// Call `leave` once when this part of the application is done with
// the channel. The underlying channel is destroyed only when the last
// ref-counted reference has been returned.
leave();
```

## **TypeScript**

The recommended way to type presence and custom messages is the global `EmberLink` augmentation, which propagates the types to every API in every SDK without explicit generic parameters. See [Concepts → Type augmentation](/concepts/type-augmentation) for the full pattern.

```typescript copyButton
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
export {};
```

## **API**

### **createClient**

```typescript
function createClient<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
>(options: CreateClientOptions): EmberClient<P, C>;
```

Creates a new `Client` configured with the provided options. The returned object exposes `joinChannel` and `destroy`. See [Concepts → Client](/concepts/client) for the full description of the lifecycle.

### **CreateClientOptions**

```typescript
interface CreateClientOptions {
	baseUrl: string;
	authEndpoint?: AuthEndpoint;
	multiTenant?: { tenantId: string };
	polyfills?: { websocket?: IWebSocket };
	jwtSignerPublicKey?: string;
}
```

| **Option**             | **Type**                       | **Required** | **Description**                                                                                                                                              |
| ---------------------- | ------------------------------ | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `baseUrl`              | `string`                       | yes          | Base URL of the Ember Link server. Used for both authentication requests and the WebSocket URL.                                                              |
| `authEndpoint`         | `string \| AuthCallback`       | no           | A URL the SDK `POST`s to with `{ channelName }`, or an async callback that returns `{ token }` directly.                                                     |
| `jwtSignerPublicKey`   | `string`                       | no           | PEM-formatted RSA public key the SDK uses to verify the signature of returned tokens. Required whenever `authEndpoint` is set.                               |
| `multiTenant`          | `{ tenantId: string }`         | no           | Sends `tenant_id` on every request. Pair with a server running in multi-tenant mode.                                                                         |
| `polyfills`            | `{ websocket?: IWebSocket }`   | no           | Drop-in replacement for the global `WebSocket` constructor, for environments that do not expose one.                                                         |

### **EmberClient**

```typescript
interface EmberClient<P, C> {
	joinChannel: JoinChannel<P, C>;
	destroy: () => void;
}

type JoinChannel<P, C> = <S extends IStorageProvider>(
	channelName: string,
	options?: ChannelConfig<S, P>['options']
) => { channel: Channel<P, C>; leave: () => void };
```

| **Method**     | **Description**                                                                                                                                              |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `joinChannel`  | Joins the named channel and returns a `{ channel, leave }` pair. Repeated calls with the same name return the same underlying channel, ref-counted internally. |
| `destroy`      | Tears down every channel the Client has created and closes their WebSocket connections.                                                                       |

### **ChannelConfig['options']**

The second argument of `joinChannel`.

```typescript
{
	initialPresence?: P;
	storageProvider?: IStorageProvider;
	autoConnect?: boolean;
	presenceThrottle?: number;
}
```

| **Option**          | **Type**             | **Default** | **Description**                                                                                                       |
| ------------------- | -------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------- |
| `initialPresence`   | `P`                  | none        | Presence value sent on join, before any explicit `updatePresence` call.                                              |
| `storageProvider`   | `IStorageProvider`   | none        | Enables shared CRDT storage on the channel.                                                                           |
| `autoConnect`       | `boolean`            | `true`      | When `false`, no WebSocket connection is opened until you call `channel.connect()`.                                   |
| `presenceThrottle`  | `number` (ms)        | `0`         | Coalesces presence sends. See [Concepts → Presence](/concepts/presence) for the throttling details and recommended values. |

### **Channel**

The handle returned inside `joinChannel`'s `{ channel }`.

```typescript
type Channel<P, C> = {
	updatePresence: (state: P) => void;
	sendCustomMessage: (data: C) => void;
	hasStorage: () => boolean;
	getStorage: () => IStorage;
	getStatus: () => Status;
	getOthers: () => User<P>[];
	getName: () => string;
	getPresence: () => P | null;
	destroy: () => void;
	connect: () => void;
	updateYDoc: (data: StorageUpdateMessage) => void;
	syncYDoc: (data: StorageSyncMessage) => void;
	events: Observable<ChannelEvents<P, C>> & {
		others: Observable<OtherEvents<P>>;
		yjsProvider: Observable<YjsProviderEvents>;
	};
};
```

| **Member**            | **Type**                                  | **Description**                                                                                                       |
| --------------------- | ----------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `updatePresence`      | `(state: P) => void`                      | Replaces the local client's presence with `state` and broadcasts the new value.                                       |
| `sendCustomMessage`   | `(data: C) => void`                       | Sends a fire-and-forget message to every other peer connected to the channel.                                         |
| `hasStorage`          | `() => boolean`                           | Returns `true` when a `storageProvider` is configured on the channel.                                                 |
| `getStorage`          | `() => IStorage`                          | Returns the channel's `IStorage`. Throws when `hasStorage()` is `false`.                                              |
| `getStatus`           | `() => Status`                            | Returns the current WebSocket status.                                                                                  |
| `getOthers`           | `() => User<P>[]`                         | Returns the current list of other peers in the channel.                                                                |
| `getName`             | `() => string`                            | Returns the channel name.                                                                                              |
| `getPresence`         | `() => P \| null`                         | Returns the local client's last-sent presence, or `null` when none has been sent yet.                                  |
| `destroy`             | `() => void`                              | Tears down the channel, bypassing ref-counting. Prefer `leave()` from `joinChannel`'s return.                          |
| `connect`             | `() => void`                              | Opens the WebSocket. Only required when `autoConnect: false` was passed.                                              |
| `events`              | `Observable<ChannelEvents<P, C>>`         | Observable for channel events. The `.others` sub-observable fires per peer.                                            |

### **Status**

```typescript
type Status = 'initial' | 'connecting' | 'connected' | 'reconnecting' | 'closed' | 'disconnected';
```

### **ChannelEvents**

The shape of the `channel.events` observable.

```typescript
type ChannelEvents<P, C> = {
	presence: (self: P) => void;
	status: (status: Status) => void;
	others: (others: User<P>[]) => void;
	destroy: () => void;
	customMessage: (message: C) => void;
};
```

### **OtherEvents**

The shape of the `channel.events.others` sub-observable.

```typescript
type OtherEvents<P> = {
	join: (user: User<P>) => void;
	leave: (user: User<P>) => void;
	update: (user: User<P>) => void;
	reset: () => void;
};
```

### **User**

```typescript
type User<P> = P & { clientId: string };
```

`User<P>` is the presence object with a `clientId` field added. The presence fields appear directly on the user object (for example `user.cursor`), rather than being nested under a `user.presence` key.

## **Related**

- [Concepts → Client](/concepts/client) for the lifecycle and authentication flow.
- [Concepts → Channels](/concepts/channels) for ref-counting and the full event model.
- [@ember-link/react](/packages/react) and [@ember-link/svelte](/packages/svelte) for framework bindings.

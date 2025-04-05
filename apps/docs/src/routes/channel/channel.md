# **Channel** API Reference

The `Channel` object represents a real-time communication channel in the Ember Link platform. It provides functionalities for presence tracking, storage synchronization, managing users, and interacting with WebSocket connections.

---

## Connecting to a Channel

To create a new instance of a `Channel`, you need to call the `joinChannel` function on a [Client](/client)

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

### Methods

```typescript
type Channel<
	P extends Record<string, unknown> = DefaultPresence,
	C extends Record<string, unknown> = DefaultCustomMessageData
> = {
	updatePresence: (state: P) => void;
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

| **Name**         | **Type**                               | **Description**                                                                 |
| ---------------- | -------------------------------------- | ------------------------------------------------------------------------------- |
| `updatePresence` | `(state: P) => void`                   | Updates the local users presence in the channel.                                |
| `hasStorage`     | `() => boolean`                        | Returns true if the channel has a storage provider, false otherwise.            |
| `getStorage`     | `() => IStorage`                       | Retrieves the channel's storage provider.                                       |
| `getStatus`      | `() => Status`                         | Returns the current connection status of the channel.                           |
| `getOthers`      | `() => User<P>[]`                      | Retrieves the list of other users in the channel.                               |
| `getName`        | `() => string`                         | Returns the name of the channel.                                                |
| `getPresence`    | `() => P`                              | Returns the presence state of the channel or null if unavailable.               |
| `destroy`        | `() => void`                           | Destroys the channel, cleaning up resources and connections.                    |
| `updateYDoc`     | `(data: StorageUpdateMessage) => void` | Sends an update to the Yjs document in the channel.                             |
| `syncYDoc`       | `(data: StorageSyncMessage) => void`   | Synchronizes the Yjs document in the channel.                                   |
| `events`         | `Observable<ChannelEvents<P, C>>`      | Event emitter for channel-specific events. Includes `others` and `yjsProvider`. |

### ChannelEvents

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
```

| **Event Name**  | **Type**                      | **Description**                                            |
| --------------- | ----------------------------- | ---------------------------------------------------------- |
| `presence`      | `(self: P) => void`           | Emitted when the presence state of the local user changes. |
| `status`        | `(status: Status) => void`    | Emitted when the connection status of the channel changes. |
| `others`        | `(others: User<P>[]) => void` | Emitted when the list of users in the channel changes.     |
| `destroy`       | `() => void`                  | Emitted when the channel is destroyed.                     |
| `customMessage` | `(message: C) => void`        | Emitted when a custom message is received in the channel.  |

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

channel.events.subscribe('other', (others) => {
	console.log('Users online:', others);
});

channel.updatePresence({ isTyping: true });
// Call leave when you are done with the channel
leave();
```

Notes

- The updatePresence method allows you to update your own presence state (such as your username or status).

- The getStorage and hasStorage methods help interact with a configured storage provider for syncing data.

- The events property provides observables for listening to channel events, including presence, status changes, and Yjs document updates.

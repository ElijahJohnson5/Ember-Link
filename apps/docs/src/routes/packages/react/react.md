# **@ember-link/react**

`@ember-link/react` is the official React integration for Ember Link. The package re-exports every type and runtime export from `@ember-link/core` and adds two context providers and a set of React hooks. The hooks handle subscription, cleanup, and reactivity so application code does not need to call `channel.events.subscribe` manually.

## **Installation**

```sh copyButton
yarn add @ember-link/react
```

## **Basic Usage**

```tsx copyButton
import {
	EmberLinkProvider,
	ChannelProvider,
	useChannel,
	useOthers,
	useMyPresence,
	useCustomMessage
} from '@ember-link/react';
import { useEffect } from 'react';

function App() {
	return (
		<EmberLinkProvider baseUrl="http://localhost:9000">
			<ChannelProvider channelName="test" options={{ presenceThrottle: 33 }}>
				<Page />
			</ChannelProvider>
		</EmberLinkProvider>
	);
}

function Page() {
	// `useChannel` returns the raw Channel<P, C> as an escape hatch for
	// any case where no hook exposes what the component needs.
	const channel = useChannel();
	const others = useOthers();
	const [myPresence, setMyPresence] = useMyPresence();
	const sendCustomMessage = useCustomMessage((message) => {
		console.log('Got custom message:', message);
	});

	useEffect(() => {
		setMyPresence({ online: true });
	}, [setMyPresence]);

	return (
		<>
			{others.map((other) => (
				<div key={other.clientId}>{other.clientId}</div>
			))}
		</>
	);
}
```

## **Providers**

### **EmberLinkProvider**

```typescript
<EmberLinkProvider {...CreateClientOptions}>
	{children}
</EmberLinkProvider>
```

Creates an Ember Link `Client` from the props passed to the provider and exposes it on React context. The props are the same as `CreateClientOptions` from `@ember-link/core` (see [@ember-link/core → Client Options](/packages/core)). The provider memoises the options so a parent that passes a shallow-equal literal on each render does not re-create the client. The client is destroyed on unmount.

### **ChannelProvider**

```typescript
<ChannelProvider channelName={string} options={ChannelOptions}>
	{children}
</ChannelProvider>
```

Joins the named channel through the Client supplied by `EmberLinkProvider` and exposes it on React context. The `options` prop is the same options object accepted by `client.joinChannel`. The channel is left (and ref-counted-down) on unmount and when `channelName` or `options` change.

## **Hooks**

### **useClient**

```typescript
function useClient<P, C>(): EmberClient<P, C>;
function useClientOrNull<P, C>(): EmberClient<P, C> | null;
```

Returns the `EmberClient` from the nearest `EmberLinkProvider`. The non-`OrNull` form throws when called outside a provider.

### **useChannel**

```typescript
function useChannel<P, C>(): Channel<P, C>;
function useChannelOrNull<P, C>(): Channel<P, C> | null;
```

Returns the raw `Channel<P, C>` from the nearest `ChannelProvider`. Useful for accessing methods not exposed by a dedicated hook, such as `channel.events.others.subscribe('join', ...)`.

### **useOthers**

```typescript
function useOthers<P, C>(): User<P>[];
```

Returns the array of other peers currently connected to the channel. The returned array updates when peers join, leave, or update their presence. Each entry has the shape `P & { clientId: string }`, so presence fields appear directly on the user object.

### **useMyPresence**

```typescript
function useMyPresence<P, C>(): readonly [P | null, (next: P) => void];
```

Returns a tuple of the local client's presence and a setter that broadcasts the new presence to the channel. The setter is stable across renders, so it is safe to pass to a child component or to use in a `useEffect` dependency array. `updatePresence` replaces the previous presence value in its entirety.

### **useStatus**

```typescript
function useStatus<P, C>(): Status;
```

Returns the current WebSocket status of the channel. See `Status` in [@ember-link/core](/packages/core) for the full set of values.

### **useCustomMessage**

```typescript
function useCustomMessage<P, C>(
	callback: (message: C) => void
): (data: C) => void;
```

`useCustomMessage` handles both directions of the custom message channel. The `callback` argument runs on every incoming custom message. The returned function sends an outgoing custom message. Both directions can be used independently. To consume only, pass a callback and ignore the return value. To send only, pass a callback that does nothing.

### **useArrayStorage**

```typescript
function useArrayStorage<T>(name: string): ArrayStorageHookResult<T>;
```

Returns a CRDT-backed array synced across all peers in the channel. The returned object includes every method of `ArrayStorage<T>` (`push`, `insertAt`, `delete`, `replace`, `toArray`, `forEach`, `subscribe`) plus a `current` field that holds the latest snapshot as a plain `Array<T>` for use in render output. The channel must have been joined with a `storageProvider` configured. See [@ember-link/storage](/packages/storage) for the underlying type.

```tsx copyButton
const items = useArrayStorage<{ id: string }>('items');
items.push({ id: crypto.randomUUID() });
// In render output:
items.current.map((item) => <li key={item.id}>{item.id}</li>);
```

### **useMapStorage**

```typescript
function useMapStorage<K extends string, V>(name: string): MapStorageHookResult<K, V>;
```

Returns a CRDT-backed map synced across all peers in the channel. The returned object includes every method of `MapStorage<K, V>` (`get`, `set`, `has`, `delete`, `clear`, `entries`) plus a `current` field that holds the latest snapshot as a plain `Map<K, V>` for use in render output.

```tsx copyButton
const meta = useMapStorage<string, string>('meta');
meta.set('title', 'My document');
const title = meta.current.get('title');
```

## **Per-Library Type Scoping**

If you are writing a library on top of Ember Link and you do not want to require your consumers to augment the global `EmberLink` interface, use `createEmberLinkContext<P, C>()`. The factory returns a typed bundle with the same `EmberLinkProvider`, `ChannelProvider`, and hook set, scoped to the `P` and `C` you pass in.

```typescript copyButton
import { createEmberLinkContext } from '@ember-link/react';

const {
	EmberLinkProvider,
	ChannelProvider,
	useOthers,
	useMyPresence,
	useCustomMessage,
	useArrayStorage,
	useMapStorage,
	useStatus,
	useChannel
} = createEmberLinkContext<
	{ cursor: { x: number; y: number } | null },
	{ kind: 'ping' }
>();
```

The runtime functions returned by `createEmberLinkContext` are the same functions as the top-level exports. Only the types differ.

## **Related**

- [@ember-link/core](/packages/core) for the underlying `Channel`, `Client`, and event types.
- [@ember-link/svelte](/packages/svelte) for the Svelte integration.
- [Concepts → Type augmentation](/concepts/type-augmentation) for the global augmentation pattern.
